# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; Nyquist alias-coupling ablation frozen
Owner: dsp
Updated: 2026-07-11
Related contracts: `046`, `048`, `049`
Related architecture: `docs/architecture/offline-time-stretch-synthesis.md`

## Purpose

Freeze the synthesis-policy boundary after the first multi-output structural
hybrid failed its mono gate. This contract governs algorithm proof work. It
does not promote a product path.

## Rules

### Rule 1: components are additive, not alternative branches

The successor owns one monotonic source-to-output time map. Harmonic, residual,
and percussive components are a complementary decomposition of one source, not
alternative full-band renders. They may use specialized processors, but every
processor receives the same fixed ratio and exact target length. Final output
is their sample-aligned sum. No ownership crossfade, branch switching, delay
alignment, local time-map change, or component gain matching is allowed.

### Rule 2: separation is iterative H/R/P

The report-only separator accepts sample-rate metadata and performs two
centred, padded STFT decompositions:

1. Extract clearly harmonic bins from the input with a frame duration nearest
   `186 ms`, quarter-window hop, separation factor `beta_h=2`, and median spans
   nearest `200 ms` horizontally and `500 Hz` vertically.
2. Apply the same rule to the first-stage complement with a frame duration
   nearest `11.6 ms`, quarter-window hop, and `beta_p=2`. Clearly percussive
   bins become the percussive component. Everything else becomes residual.

Supported FFT sizes are powers of two. Magnitude-median boundaries replicate
the nearest valid frame or bin. Signal boundaries use the existing centred zero
padding and normalized overlap-add policy.

The binary masks are disjoint. At every time-frequency bin,
`M_h + M_r + M_p = 1`. Each masked spectrum retains the source complex phase.
No learned separator, soft-mask sweep, classifier guard, or post-separation
gain correction enters the first proof.

### Rule 3: separation must pass before component TSM

Batch 29.6D proves decomposition and reconstruction only. Harmonic, residual,
and percussive time-domain components must sum back to the exact source-domain
render within the declared numerical tolerance. No component is stretched in
this batch.

Synthetic controls must assign a steady bin-centred sinusoid primarily to the
harmonic component, an isolated broadband impulse primarily to the percussive
component, and stationary broadband noise primarily to the residual. The
expected owner must exceed either specialized non-owner by at least `12 dB`.
Failure rejects the separator before a corpus TSM render.

### Rule 4: component processing is fixed

Only after Batch 29.6D passes may Batch 29.6E apply:

- long-window identity-locked phase-vocoder TSM to the harmonic component
- the current `2048/512` OfflineHighQuality kernel to the residual component
- plain normalized OLA, using the short separation frame and quarter-window
  analysis hop, to the percussive component

OLA performs no waveform search, onset detection, transient reinsertion, phase
reset, or local timing compensation. Each component independently produces the
same target length from the same global ratio before sample-aligned addition.

### Rule 5: exactness and evidence remain mandatory

Every proof retains identity bypass, deterministic output, finite samples,
centred boundary coverage, exact target length, and explicit mapping evidence.
The separator reports mask population, partition error, component energy,
reconstruction RMS/peak error, endpoint error, and synthetic ownership. The
TSM proof adds component output lengths, component peak growth, transient
replica ratio, recombination peak growth, and current-versus-candidate quality.

### Rule 6: promotion stays closed

Production routing, cache identity, product receipts, pitch composition,
dynamic-ratio routing, RealtimePreview, and linked stereo remain unchanged.
Batch 29.6D passed on 2026-07-10. Batch 29.6E failed the frozen mono gate and
is rejected without tuning. Batch 29.7 remains closed.

### Rule 7: the active successor is one whole-band phase-gradient transform

Batch 29.6F supersedes component synthesis with one fixed-resolution STFT. It
estimates both time- and frequency-direction partial derivatives of analyzed
phase and integrates them through a magnitude-prioritized max heap. It does not
use peak tracking, onset detection, phase reset, source separation, component
TSM, waveform search, adaptive resolution, or local ratio compensation.

Freeze the first proof to a `4092`-sample Hann analysis window, `8192`-point
FFT, `1024`-sample synthesis hop, and analysis hop equal to
`round(1024 / ratio)`. Ratios remain fixed per render. The existing centered
zero-padding, normalized overlap-add, exact target crop, and bit-exact identity
bypass remain authoritative.

### Rule 8: derivative and integration policy is explicit

Use heterodyned backward and forward time-phase differences and average them
for the centered time derivative. Use backward and forward wrapped
frequency-phase differences and average them for the centered frequency
derivative. Time and frequency propagation both use trapezoidal integration.

The first synthesis frame copies analyzed phase. Offline analysis supplies the
previous/current/future frames required by centered time differences. Operate
on DC through Nyquist, use one-sided frequency differences at those two
boundaries, then mirror the nonredundant synthesis coefficients to enforce
conjugate symmetry.

For each later frame, set `abstol` to `1e-6` times the maximum magnitude across
the previous and current frames. Significant current bins are assigned exactly
once. Seed the max heap with significant bins from the previous frame, then
propagate from the highest-magnitude available predecessor in time or frequency
until no significant current bin remains. Heap ordering must include stable
frame/bin tie breaks. Bins at or below `abstol` copy analyzed phase rather than
use random phase.

### Rule 9: prove the kernel before the corpus candidate

Batch 29.6F is a report-only mechanism proof. It covers a steady bin-centered
sine, linear chirp, isolated broadband impulse, two simultaneous sines, silence,
and repeated identical input. Report derivative finiteness, significant and
insignificant bins, horizontal and vertical assignments, duplicate or missing
assignments, heap high-water mark, conjugate-symmetry error, overlap-add
coverage, exact output length, and deterministic hashes.

The proof passes only when:

- every reported derivative is finite
- every significant current bin is assigned once and only once
- no heap operation exceeds the declared frequency-bin bound
- synthesized spectra are conjugate symmetric within `1e-6`
- no output sample is non-finite or uncovered
- output length equals the exact target length
- repeated inputs produce identical traces, hashes, and samples
- horizontal propagation occurs for the steady sine and both tones
- vertical propagation occurs for the chirp and impulse

These synthetic ownership checks prove that both integration directions are
live. They do not claim sound-quality promotion.

### Rule 10: the complete mono gate remains separate

Batch 29.6G may render the 60-row corpus only after Batch 29.6F passes. It must
report every existing Batch 29.6 quality and integrity field, compare against
the current kernel and external comparator, and pass the unchanged complete
mono gate before linked stereo opens. Do not sweep geometry, tolerance,
derivative policy, or heap priority inside that gate.

### Rule 11: exact requested mapping precedes new transient policy

Batch 29.6H retains the whole-band phase-gradient core but replaces the
rejected repeated rounded analysis hop. For rendered frame `n`, define the
unpadded analysis centre as:

`A_n = round(n * 1024 / ratio)`

Adjacent analysis intervals are `A_n - A_(n-1)`. They must be positive and may
differ only between the floor and ceiling of `1024 / ratio`. Generate absolute
positions directly; do not accumulate floating-point positions or repair the
final interval.

Each backward and forward heterodyned time-phase difference uses its own actual
analysis interval in both the nominal phase advance and derivative divisor.
The centered time derivative remains the average of those two instantaneous
frequency estimates. Synthesis hop remains `1024`; frequency-direction
integration uses the requested global ratio.

Window `4092`, FFT `8192`, tolerance `1e-6`, heap ordering, first-frame phase,
insignificant-bin phase, conjugate mirroring, padding, normalization, exact
target crop, and identity bypass do not change. No onset detector, phase reset,
transient gain, envelope correction, source separation, waveform search, or
local time compensation enters this proof.

### Rule 12: exact-lattice evidence and stop gate

Before corpus rendering, report ideal and integer analysis positions, interval
floor/ceiling counts, maximum absolute mapping error, monotonicity, final
mapping error, phase assignments, heap bound, symmetry, coverage, and hashes.
The mapping proof requires:

- every integer analysis centre is within `0.5` frame of its ideal position
- every interval is positive and equals floor or ceiling of the ideal interval
- positions and intervals are deterministic and monotonic
- zero missing or duplicate significant-bin assignments
- unchanged finite-output, symmetry, coverage, exact-length, and identity gates

After the mapping proof passes, run the unchanged 60-row complete mono gate.
Retain the Batch 29.6G comparator fields. The exact-lattice candidate must pass
the entire gate before linked stereo opens; timing improvement alone is not
promotion. Failure returns to research without hop, phase, or parameter tuning.

## Separation Proof Gate

- masks are binary, mutually exclusive, and exhaustive for every analyzed bin
- component lengths equal input length
- recombined source peak error is at most `1e-5`
- recombined source RMS error is at most `1e-6`
- no non-finite component sample, uncovered source sample, or endpoint loss
- harmonic, percussive, and residual synthetic controls each meet the `12 dB`
  ownership margin
- identical input, sample rate, and parameters produce identical components

Batch 29.6D passes this gate. At `48 kHz`, the frozen geometry resolves to
`8192/2048` long analysis and `512/128` short analysis. The mixed reconstruction
control measured `8.940697e-8` peak error and `1.939046e-8` RMS error with zero
uncovered source samples. Ownership margins were `30.933980 dB` for the steady
sine, `164.871272 dB` for the isolated impulse, and `12.925746 dB` for stationary
noise. Repeated component vectors and hashes were identical.

## Additive Mono TSM Gate

- improve anchored `L001` crest by at least `3 dB`
- keep the candidate worst crest at or below `5.655483 dB`
- do not worsen corpus mean absolute event placement by more than `1` frame
- retain `60/60` integrity, transient, formant, boundary, and combined passes
- do not regress source-relative residual or unsupported-bin mass
- retain the original Batch 29.6 fast spectral-movement gate
- do not worsen the strongest post-attack secondary-peak/primary-peak ratio by
  more than `0.10` within one short percussive frame
- no non-finite output, non-monotonic synthesis position, uncovered output
  sample, component length mismatch, or hidden component gain correction

This gate proves the complete fixed-ratio additive mono mechanism. It does not
promote product routing or waive independent listening and linked-stereo gates.

## 2026-07-10 Additive H/R/P Proof Outcome

The additive candidate improved anchored `L001` crest by `3.375261 dB`, kept
worst crest to `4.083747 dB`, and reduced mean fast spectral movement at both
expansion ratios. It nevertheless failed the complete gate: measurable-row
mean event placement worsened `23.411637` frames, integrity passed `51/60`,
post-attack replica protection passed `26/48`, static residual and unsupported
bin mass regressed at both expansion ratios, and the combined gate passed
`0/60`.

Do not tune masks, separation factors, component gains, processor geometry, or
component timing. Do not open linked stereo.

## 2026-07-10 Proof Outcome

The adaptive transient timeline failed: `L001` improved only `0.536217 dB`,
mean event placement worsened by `4.942263` frames, and the combined gate passed
`9/60`. Exact anchors and overlap-add coverage passed, but sparse onset anchors
required local hops up to `1664` frames and moved unprotected events. Do not
tune classifier or compensation constants and do not open adaptive resolution.

## 2026-07-10 Reassessment Decision

Use peak-local group-delay phase reinitialization under the unchanged global
time map for the next proof. This mechanism targets invalid transient phase
prediction and broad phase ownership inside the existing STFT kernel without
moving unrelated events.

Do not implement explicit transient/residual separation in this proof. That
branch requires a new multiresolution perfect-reconstruction split, adaptive
mask continuity, separate component processing, and recombination policy. It
also exposes threshold leakage and synthetic-component artifacts before the
smaller in-engine mechanism has been tested. Separation remains a research
fallback if the fixed-map peak proof fails its frozen gate.

## 2026-07-10 Fixed-Map Peak Proof Outcome

The fixed-map peak proof failed. Anchored `L001` crest improved only
`0.040942 dB`, measurable-row mean event placement worsened `16.851522`
frames, and the combined gate passed `12/60`. Integrity, added silence, peak
growth, and overlap-add coverage passed `60/60`, but `984/2370` guarded events
never reached a reported centre-threshold reset. Tonal residual regressed in
`21/60` rows and unsupported-bin mass regressed in `24/60`.

Do not tune the window-derived threshold, sensitivity, event guards, or reset
scope. Do not open adaptive resolution or linked stereo.

## 2026-07-10 H/R/P Reassessment Decision

The next proof uses refined harmonic/residual/percussive separation. The
residual component is mandatory: two-way H/P processing is known to route
ambiguous harmonic material such as voice into the short OLA path, where phase
jumps become audible. Iterative long/short separation and `beta=2` isolate only
clearly harmonic or percussive structures while preserving a complementary
residual.

This additive structure does not reopen the rejected full-band branch
crossfade. Component reconstruction is proven before TSM, and every component
uses the same output map and target length.

## 2026-07-10 Full Phase-Gradient Reassessment Decision

The additive H/R/P candidate failed timing, integrity, transient-replica,
static-spectrum, and combined gates despite passing source separation and crest
checks. Independent component synthesis is therefore closed without tuning.

WSOLA, sinusoidal/residual synthesis, and onset-compensated adaptive-resolution
methods do not provide a sufficiently clean next boundary. The active proof is
full phase-gradient integration inside one whole-band STFT. It targets the
current kernel's neglected frequency-direction phase structure without source
separation or local time redistribution. The published method reports
competitive listening results against commercial universal-mode systems, but
Signal must first prove its own deterministic kernel and complete corpus gate.

## 2026-07-10 Full Phase-Gradient Kernel Outcome

Batch 29.6F passes. The report-only kernel uses the frozen `4092`-sample Hann
window, `8192` FFT, `1024` synthesis hop, ratio-derived analysis hop, centered
time and frequency differences, and deterministic magnitude-prioritized heap.

At `1.5x`, every sine, two-tone, chirp, and impulse significant bin received
one phase assignment. Missing and duplicate assignments were zero. Heap
high-water was `4098` or `4099` entries against the `8194` bound. The steady
sine used `17104` horizontal and `18310` vertical assignments; the two-tone
control used `20673` and `27422`; the chirp used `39128` and `64065`; the
impulse used `8198` and `16384`. Silence, `0.75x` compression, bit-exact
identity, exact length, finite derivatives/output, conjugate symmetry,
overlap-add coverage, and repeat hashes all passed.

This proves the mechanism only. It does not pass the complete mono quality gate
or promote product routing.

## 2026-07-10 Full Phase-Gradient Mono Outcome

Batch 29.6G is rejected without tuning. All `60` rows retained exact phase
assignment, heap bounds, conjugate symmetry, finite output, exact length, and
overlap-add coverage. Added silence and peak-growth limits passed `60/60`;
complete endpoint integrity passed `57/60`.

The whole-band kernel improved the strongest structural evidence left by the
H/R/P failure. Tonal regression-free passed `55/60`. At `1.25x` and `1.5x`,
mean spectral-modulation delta improved by `-0.003056500` and `-0.002028650`,
source-relative residual improved by `-0.034376250` and `-0.039958950`, and
unsupported-bin mass improved by `-0.001094150` and `-0.001203250`. Against
Rubber Band, mean aligned correlation rose from `0.327354900` for the current
kernel to `0.367353969`; mean RMS error fell from `0.187637615` to
`0.166064781`.

The complete gate still failed. Anchored `L001` crest improved only
`1.667930 dB`; required improvement was `3 dB`. Measurable event placement
worsened by `16.738760` frames on average and up to `137` frames. The
post-attack replica gate passed `28/48`, with worst ratio delta `+0.675459`.
Transient, formant, boundary, and combined gates passed `18/60`, `10/60`,
`53/60`, and `3/60`. Worst candidate crest was `4.103372 dB`, within the
`5.655483 dB` limit.

Do not tune window geometry, tolerance, derivative policy, heap priority, or
boundary crop. Do not open linked stereo.

## 2026-07-10 Exact-Lattice Reassessment Decision

The rejected candidate repeated `round(1024 / ratio)` as one constant analysis
hop. Its actual lattice ratios differ from the requested ratios and can drift
roughly `40`, `67`, and `161` source-mapped frames over the five-second corpus
at `0.75x`, `1.25x`, and `1.5x`. Exact output cropping does not correct event
positions inside that render.

Public phase-vocoder equations propagate phase between arbitrary analysis and
synthesis frame centres using their actual adjacent differences. Batch 29.6H
therefore tests an absolute rounded analysis-centre schedule before adding any
new attack or shape mechanism. This preserves the successful whole-band tonal
policy while removing a measured mapping confound.

Speech-specific shape-invariant processing remains deferred because it adds
sinusoidal/noise classification, correlation, envelope, and balance policy.
Peak-local transient phase reinitialization remains rejected by Batch 29.6C.

## 2026-07-10 Exact-Lattice Proof Outcome

Batch 29.6H is rejected without tuning. Mapping passed `60/60` with maximum
analysis-centre error `0.4` frame, but the complete gate did not improve enough:
`L001` crest improvement was `2.379387 dB`, timing worsened `17.789744` frames
on average and `151.25` frames worst-case, integrity passed `57/60`, replica
protection passed `27/48`, and combined passed `3/60`. Tonal regression-free
improved to `57/60`; expansion residual, unsupported-bin, and fast-movement
means remained better than the current kernel.

Exact lattice removes a real mapping error but does not explain the dominant
event-placement defect. Do not tune its schedule or reopen linked stereo.

### Rule 13: the next transform is frequency-adaptive and painless

Batch 29.6I replaces the fixed-resolution STFT only in a report-only transform
proof. Construct one frequency-adaptive nonstationary Gabor frame with:

- logarithmically spaced constant-Q interior bands
- explicit DC and Nyquist completion bands
- compact frequency-domain analysis filters
- per-band decimation no coarser than the painless-frame support condition
- canonical dual filters computed from the strictly positive diagonal frame
  operator

The filter lattice and duals are immutable for one proof render. All bands
belong to one analysis/synthesis system. Independent time-domain branch
renders, crossovers, source masks, onset-adaptive windows, local ratio changes,
and output recombination remain prohibited.

### Rule 14: reconstruction truth precedes time stretching

Batch 29.6I performs no time stretch and no phase modification. It must report:

- minimum and maximum diagonal frame-operator values and their finite positive
  condition ratio
- band count, centre frequency, support length, decimation, and coefficient
  count
- uncovered and multiply assigned frequency samples
- source/output length, peak error, RMS error, endpoint error, and non-finite
  counts
- per-band impulse peak position relative to the declared common origin
- repeat hashes for filters, coefficients, and reconstruction

Sine controls spanning low, crossover, high, DC-near, and Nyquist-near bands,
a broadband impulse, deterministic noise, mixed tonal/transient content, and
silence must pass. Peak reconstruction error is at most `1e-5`; RMS error is at
most `1e-6`; lengths and endpoints are exact; all frame-operator samples are
finite and strictly positive; impulse delay agrees with the declared origin to
within one sample; and repeated reports and samples are identical.

Failure rejects the transform geometry. Passing opens only a separately
contracted frequency-adaptive phase-gradient mechanism proof. It does not open
the 60-row corpus, linked stereo, dynamic ratio, cache identity, or product
routing.

## 2026-07-10 Frequency-Adaptive Reconstruction Outcome

Batch 29.6I passes without geometry tuning. The `4096`-frame mixed control used
`576` bands and `10634` coefficients. Frame bounds were `0.999999881` and
`1.000000119`, condition ratio was `1.000000238`, and all `4096` frequency bins
were covered. Peak and RMS reconstruction error were `1.490116119e-7` and
`3.762034804e-8`. Every compact support satisfied its decimation, band delay
was zero, samples and coefficients were finite, and repeat hashes matched.

This proves transform geometry only. Filter-bank phase derivatives,
cross-band integration topology, coefficient-time mapping under stretch, and
real-output symmetry remain undefined. Do not infer them from the passing
identity proof.

### Rule 15: unequal band lattices do not authorize phase propagation

Batch 29.6I uses one decimation per band. It proves efficient painless-frame
reconstruction but does not provide time-aligned rows. Do not interpolate rows
onto an implicit grid, choose nearest cross-band coefficients, or apply the
fixed-STFT heap to unequal time positions.

Published filter-bank PGHI assumes uniform decimation. Its authors identify a
truly nonuniform heap as future work and describe filter-bank time stretching
only as conceivable. That source does not authorize Signal to invent the
missing topology inside a corpus candidate.

### Rule 16: prove one common-grid frequency-adaptive frame

Batch 29.6J replaces only the Batch 29.6I proof geometry. Use the published
grid-decimated wavelet configuration with:

- analytic Cauchy mother wavelet with `alpha=900`
- `1536` nonnegative-frequency channels, including `16` lowpass completion
  channels
- uniform `384`-frame decimation, giving redundancy `8`
- deterministic digital `(0,1)` channel delays from the published generator
- canonical dual synthesis from the complete uniform-filter-bank frame
  operator

At `48 kHz`, channel centres are uniformly spaced from DC to Nyquist by
`15.625 Hz`; wavelet bandwidth increases with centre frequency. The proof is
offline, report-only, and identity-only. It performs no phase modification or
stretch.

Report channel count, lowpass count, hop, redundancy, delay-sequence hash,
minimum/maximum frame bounds, condition ratio, dual residual, analysis and
synthesis coefficient counts, reconstruction peak/RMS/endpoint error,
non-finite values, source/output hashes, and repeat hashes.

Run the Batch 29.6I sine, edge, impulse, noise, mixed, silence, and empty-input
controls. Require condition ratio at most `1.25`, canonical-dual residual at
most `1e-8`, peak reconstruction error at most `1e-5`, RMS error at most
`1e-6`, exact length and endpoints, finite values, and identical repeated
reports. Failure returns to research. Passing opens only a separately
contracted common-grid phase-gradient mechanism proof.

## 2026-07-10 Common-Grid Wavelet Reconstruction Outcome

Batch 29.6J passes. The `4096`-frame mixed control pads to `4224` frames and
produces `16896` coefficients on the `1536 x 11` common grid. Deterministic
frequency-response tightening precedes the complete alias-block frame solve.
Estimated frame bounds were `0.984806890` and `1.010234560`, condition ratio was
`1.025819956`, and maximum canonical-dual residual was `6.225219e-11`. Peak and
RMS reconstruction error were `2.910383e-11` and `5.520117e-13`.

All frozen sine, edge, impulse, noise, mixed, silence, empty, finite-value,
endpoint, and repeat gates pass. This authorizes only a common-grid phase
mechanism contract. It does not authorize a stretch or corpus candidate.

### Rule 17: phase lives on nominal common-grid time

For channel `k`, coefficient column `n` is analyzed by an atom centred at
`t[n,k]=n*384+d[k]`, where `d[k]` is the frozen digital `(0,1)` delay. Centered
wrapped differences along one channel estimate instantaneous angular frequency
using the actual `384`-frame interval. The delay is constant in time and does
not alter that horizontal derivative.

Transport coefficient phase to nominal time `n*384` by subtracting
`omega_hat[n,k]*d[k]` under Signal's analysis-filter convention. Compute
vertical wrapped differences only between adjacent channel centres after that
transport. Divide by the exact `15.625 Hz` centre interval. A synthetic steady
tone must prove the compensation sign before any heap integration opens.

### Rule 18: keep synthesis uniform and project source fractionally

The output canonical-dual bank retains uniform `384`-frame decimation. Output
column `m` projects to source coordinate `u=m/ratio`. Evaluate magnitudes and
both phase derivatives by bounded linear interpolation between the adjacent
source columns around `u`. Do not interpolate wrapped complex coefficients.

Source padding supplies the two derivative neighbors at each boundary. The
ideal fractional coordinate remains authoritative; no repeated rounded
analysis or synthesis hop enters the mechanism. Exact target length is
`round(source_frames*ratio)` and output padding is cropped only after complete
canonical-dual coverage.

### Rule 19: the first phase proof is synthetic and report-only

Batch 29.6K integrates the interpolated delay-compensated gradient with one
magnitude-prioritized bounded heap over the positive-frequency common grid.
Every significant coefficient is assigned once from an adjacent time or
frequency predecessor. Insignificant coefficients use deterministic analyzed
phase. Canonical-dual synthesis mirrors positive-frequency bins explicitly.

Test identity, `0.75x`, and `1.5x` on steady low/mid/high tones, two-tone,
linear and exponential chirps, broadband impulse, deterministic noise, mixed
tonal/transient content, and silence. Report:

- maximum source-coordinate error and monotonicity
- horizontal instantaneous-frequency error on steady tones
- delay-compensated adjacent-channel phase residual
- finite derivative and interpolation counts
- significant/insignificant, horizontal/vertical, duplicate/missing assignment
  counts and heap high-water
- conjugate-symmetry error, canonical-dual residual, uncovered output samples,
  exact length, non-finite values, and repeat hashes
- impulse peak error from the exact projected position

Require source-coordinate error at most `1e-9`, monotonicity, steady-tone
angular-frequency error at most `1e-6` radians/sample, compensated phase
residual at most `2e-5` radians, no duplicate or missing significant
assignments, heap high-water within `2*1536*output_columns`, conjugate-symmetry
error at most `1e-9`, dual residual at most `1e-8`, no uncovered or non-finite
output, exact target length, impulse peak error at most one frame, and identical
repeat hashes. Failure returns to research. Passing opens the unchanged 60-row
mono corpus gate; it does not open stereo or product routing.

## 2026-07-10 Common-Grid Phase-Transport Outcome

Batch 29.6K stops at its first tone gate. `312.5 Hz` and `1 kHz` controls pass
delay compensation with maximum angular-frequency error `1.478986e-7`. At
`8 kHz`, horizontal phase differences alias: angular-frequency error is
`0.065450362` radians/sample and compensated adjacent-channel residual is
`0.243248864` radians.

The `384`-frame hop permits only a `+/-62.5 Hz` unambiguous heterodyned residual
around a channel centre. High-frequency Cauchy filters are wider than that
interval. Do not implement interpolation, heap integration, synthesis, or the
corpus gate on these derivatives. The common-grid reconstruction remains valid;
the phase-difference estimator is rejected.

### Rule 20: instantaneous frequency uses a same-column derivative ratio

Batch 29.6L retains the passing Batch 29.6J filters, tightening, channel delays,
and `384`-frame coefficient grid. For finalized analysis response `G[k,w]`,
derive an auxiliary time-derivative response by multiplying with signed angular
frequency. Analyze the source through both banks at identical positions.

For qualified coefficient `C` and derivative coefficient `Cdot`, estimate
absolute instantaneous angular frequency from the imaginary part of
`Cdot*conj(C)/|C|^2`. Freeze the multiplication sign only after a synthetic
steady tone proves it. Do not combine this estimator with inter-column unwrap
or a hidden shorter hop.

Qualify coefficients at or above `0.5` of the maximum magnitude for that tone.
Within each column, choose the qualified channel with maximum coefficient
energy as the deterministic coherent carrier estimate. Apply that shared
carrier to qualified channels before removing deterministic channel delay as
in Rule 17 and measuring the strongest adjacent qualified pair at nominal
common-grid time. This dominant-carrier policy avoids combining leakage-biased
estimates from weak overlapping filters.

### Rule 21: prove the estimator before phase integration

Test periodic `312.5 Hz`, `1 kHz`, `8 kHz`, and `19.5 kHz` tones over a legal
`24576`-frame transform, plus silence and deterministic noise. Report maximum
angular-frequency error, compensated adjacent-channel residual, qualified
horizontal/vertical counts, zero-energy skips, non-finite values, auxiliary
coefficient hash, and repeat hash.

Every tone must have qualified horizontal and adjacent-channel evidence,
angular-frequency error at most `1e-6` radians/sample, and compensated residual
at most `2e-5` radians. Silence must produce no qualified values or non-finite
ratios. Noise must remain finite. Repeated evidence must match exactly.

Failure returns to research. Passing opens only the remaining fractional
projection and bounded heap mechanism proof. It does not open synthesis, the
60-row corpus, stereo, or product routing.

Batch 29.6L passes. Across the four tones, maximum angular-frequency error is
`3.614443e-12` radians/sample and maximum compensated residual is
`8.683081e-10` radians. Silence produces no qualified ratios; deterministic
noise remains finite; repeated evidence and hashes match exactly.

Rules 22 and 23 replace Rule 19's implementation staging after the rejected
phase-difference estimator. They split projected-field integration from audio
synthesis; Rule 19's unchanged synthetic controls and final gates remain the
later synthesis target.

### Rule 22: project three fields at the exact source coordinate

Batch 29.6M is report-only. Keep the passing Batch 29.6J analysis geometry and
Batch 29.6L instantaneous-frequency estimator unchanged. For output column
`m`, compute `u=m/ratio` in source-column units. Linearly interpolate:

- coefficient magnitude at each channel
- absolute instantaneous angular frequency at each channel
- delay-compensated vertical phase derivative at each channel

Do not interpolate wrapped coefficient phase. Source padding supplies legal
left and right columns at both boundaries; clamp only the interpolation index,
not the authoritative fractional coordinate. Record lower/upper source-column
indices, interpolation fraction, boundary-pad reads, projected-field counts,
maximum coordinate reconstruction error, monotonicity, finite values, and a
stable projected-field hash.

Let `target_frames=round(source_frames*ratio)`. Project
`ceil(target_frames/384)+1` columns, including the terminal coverage column.

Test ratios `0.75`, `1.0`, and `1.5` on steady tones, two-tone, linear and
exponential chirps, impulse, deterministic noise, mixed tonal/transient
content, and silence. Require coordinate reconstruction error at most `1e-9`,
strict monotonicity for non-empty multi-column output, finite fields, exact
projected-column count, exercised fractional and boundary cases, and identical
repeat reports.

### Rule 23: integrate one output column with one bounded heap

Solve positive-frequency channel phases one output column at a time. Phase is
never interpolated: column zero uses the nearest source column's
delay-compensated analyzed phase as its deterministic seed, with exact halfway
ties choosing the lower column. Later columns admit two predecessor classes:

- horizontal: the same channel in the preceding solved output column, advanced
  by `384` times the trapezoidal mean projected instantaneous frequency
- vertical: the adjacent solved channel in the current output column, advanced
  by the trapezoidal mean vertical phase derivative times the exact adjacent
  centre-frequency interval `pi/1535`

Use a magnitude-prioritized max heap. Break equal-magnitude ties by horizontal
before vertical, then lower target channel, then lower predecessor channel. A
coefficient is significant when its projected magnitude exceeds `1e-6` times
that column's maximum. Assign each significant coefficient exactly once.
Insignificant coefficients retain their deterministic nearest-column analyzed
phase and may provide a boundary seed, but do not count as heap assignments.

The heap capacity is `2*1536` entries and must not scale with source or output
length. Report significant/insignificant cells, horizontal/vertical
assignments, duplicate/missing assignments, seed counts, heap high-water,
capacity, non-finite phases, assignment hash, and repeat hash. Require at least
one horizontal and one vertical assignment across the non-silent controls, no
duplicate or missing significant assignments, high-water within capacity,
finite phases, and exact repeat evidence.

Failure returns to the projection or topology contract. Passing opens only a
separately frozen canonical-dual synthesis and synthetic placement proof. It
does not open audio synthesis, the 60-row corpus, linked stereo, dynamic ratio,
or product routing.

Batch 29.6M passes all `30` control/ratio cases. It records `34592` horizontal
and `10405` vertical assignments with no duplicate or missing significant
cells. Maximum observed heap occupancy is `1756/3072`. Coordinate error is
zero, projected fields and phases are finite, boundary and fractional cases are
exercised, and repeated evidence and hashes match exactly.

### Rule 24: derive a two-sided synthesis guard from the canonical dual

Batch 29.6N must not synthesize the target from a zero-origin circular grid.
For the render's finalized canonical-dual bank, measure every positive-channel
dual atom in the time domain. Find the smallest whole `384`-frame guard whose
excluded two-sided squared energy is at most `1e-12` of total atom energy for
every channel. Add one coefficient column for the projected-field derivative
neighbor. The resulting guard must not exceed `16384` sample frames.

Report per-channel required radius, maximum tail-energy ratio, selected guard
frames/columns, limiting channel, non-finite values, dual-atom hash, and repeat
hash. Failure to find a finite passing guard within the cap returns to transform
research before coefficient assembly.

Batch 29.6N stops here. The exact complete-frame canonical dual for lowpass
channel `0` has excluded energy `6.270779e-7` at the largest legal support
radius. Its guard lower bound is `16768` frames, beyond the `16384`-frame cap.
The block-solve residual is `1.051210e-12`, all values are finite, and repeated
atom hashes match. No coefficient assembly or audio synthesis is authorized.

### Rule 25: attribute the dual-atom tail before redesign

Batch 29.6O is diagnostic-only. On the unchanged `34176`-frame probe, measure
channels `0`, `15`, `16`, `768`, and `1535`. They represent the limiting DC
lowpass, final lowpass completion, first ordinary wavelet, interior wavelet,
and Nyquist-edge wavelet.

For each channel, measure these frozen response stages:

- raw finalized analysis response before per-bin tightening
- tightened analysis response used by Batch 29.6J
- exact complete-frame canonical-dual response after tightening

For every stage, form both the positive-only analytic complex atom and the
explicit conjugate-mirrored real-output atom. Do not alter cutoff, `alpha`,
channel count, lowpass count, delays, tightening, decimation, or dual solver.

Report excluded squared-energy ratios at whole-hop radii `384`, `1536`, `4096`,
`8192`, `12288`, and `16000` frames. Also report the first whole-hop guard lower
bound for thresholds `1e-6`, `1e-8`, `1e-10`, and `1e-12`; peak position, total
energy, exact dual residual, non-finite values, per-stage/channel hashes, and a
repeat hash.

Compute fixed attribution ratios at radius `16000`:

- tightening: tightened-analysis tail / raw-analysis tail
- dualization: canonical-dual tail / tightened-analysis tail
- mirroring: real-output tail / analytic tail
- lowpass specificity: channel `0` tail / channel `16` tail, and channel `0`
  tail / channel `768` tail

Tightening, dualization, and lowpass-specific ratios use conjugate-mirrored
real-output atoms. Mirroring ratios compare real-output against analytic atoms
within every stage and channel.

Zero denominators report infinity explicitly rather than substituting an
epsilon. The diagnostic passes only when all `30` stage/channel/form atoms are
present, values are finite except declared infinite attribution ratios, dual
residual is at most `1e-8`, fixed radii and thresholds are unchanged, and
repeated evidence and hashes match exactly.

Passing does not authorize a redesign. It opens one planning checkpoint to
choose among lowpass completion, tightening, analytic boundary, dualization,
or transform-family work based on measured ownership. Failure returns to the
diagnostic implementation. Coefficient assembly, audio synthesis, corpus,
stereo, dynamic ratio, and product routing remain closed.

Batch 29.6O passes its diagnostic gate. At radius `16000`, channel `0` raw
real-output tail is `1.622121e-13`; tightening raises it to `6.270779e-7`, a
`3865790.426x` increase. Exact dualization changes that result by only
`1.000000000248x`. Channels `15`, `16`, and `768` are below numerical tail
resolution at that radius. Channel `1535` retains `1.180453e-7` before
tightening, `1.699919e-7` after tightening, and `2.030199e-7` after
dualization. Maximum dual residual is `9.524707e-11`; all atoms are finite and
repeat exactly.

The tail has two boundary owners: tightening breaks the DC real-mirror
cancellation, while the Nyquist-edge response is already long before
tightening. Do not remove tightening alone or retune the dual solver. Any next
candidate must jointly define smooth real-output DC and Nyquist completion,
then re-prove frame conditioning, reconstruction, derivative scale, projection,
and guard bounds before synthesis can reopen.

### Rule 26: boundary completion uses one untightened frame candidate

Batch 29.6P tests one geometry. Keep raw channels `0..1534` bit-identical to
the pre-tightening Batch 29.6J responses, including their centres, Cauchy
`alpha=900`, cutoff, and digital delays. Remove per-bin tightening for the
entire candidate bank. Replace only channel `1535` with a zero-delay real
Nyquist completion.

Let normalized positive frequency be `f in [0,0.5]`, spacing be
`h=0.5/1535`, completion width be `w=16h`, and `s=clamp((f-(0.5-w))/w,0,1)`.
Use cubic smoothstep `q=s^2*(3-2s)` and Nyquist magnitude
`sin(pi*q/2)`. It is zero below `0.5-w`, unity at Nyquist, and has zero first
derivative at both support endpoints. Its phase and delay are zero. Do not add
a matching pointwise normalizer, gain correction, or second boundary variant.

This candidate is not assumed tight. The complete uniform-filter-bank frame
operator and exact canonical dual own reconstruction. Smooth compact frequency
windows plus verified coverage/frame bounds follow the painless and uniform
nonstationary-Gabor construction boundary; no source implementation is copied.
Primary references: [Holighaus et al.](https://arxiv.org/abs/1210.0084) and
[Dörfler and Matusiak](https://arxiv.org/abs/1112.5262).

First prove on the unchanged Batch 29.6J reconstruction controls:

- every positive bin covered and no non-finite filter/frame values
- complete frame condition ratio at most `1.25`
- canonical-dual residual at most `1e-8`
- exact length, peak error at most `1e-5`, RMS error at most `1e-6`, and
  head/tail error at most `1e-5`
- channels `0..1534` raw-response hashes unchanged and one stable channel
  `1535` completion hash
- identical evidence and hashes on repeat

Then run the Rule 24 guard on channels `0`, `15`, `16`, `768`, `1534`, and
`1535`. Every representative dual atom must fit within `16384` frames at
`1e-12` excluded energy. Only after those pass may an all-channel guard scan
open. The all-channel scan must meet the same cap and threshold with dual
residual at most `1e-8`, finite atoms, and exact repeat evidence.

Passing the transform and all-channel guard reopens only Batch 29.6L and 29.6M
mechanism reproof on the new bank. The four tone estimator controls and all `30`
projected-field/heap cases must retain their existing gates. Audio coefficient
assembly, inverse synthesis, corpus, stereo, dynamic ratio, cache identity, and
product routing remain closed.

Batch 29.6P is rejected at the first gate. The candidate covers every positive
bin and reconstructs exactly through the complete canonical dual, but its frame
minimum `0.7361080721` and maximum `2.1937926704` produce condition ratio
`2.9802589505`, above `1.25`. Dual residual is `7.657381e-11`; peak, RMS, head,
and tail identity errors pass. Preserved-channel hash `899c7f7b775c1378` and
Nyquist-completion hash `463ca8b834c318d5` repeat exactly. The representative
guard, all-channel guard, phase reproof, and synthesis remain unauthorized.

### Rule 26A: use one endpoint-even common frame normalizer

Batch 29.6Q freezes one scalar preconditioner. Start from the complete raw Rule
26 bank, including unchanged channels `0..1534` and the channel `1535` Nyquist
completion. Let its positive-frequency energy be
`E(f)=sum_k |H_k(f)|^2` and its exact scalar tightener be
`r(f)=1/sqrt(E(f))`. Coverage failure, non-finite energy, or non-positive
energy rejects the candidate; do not clamp or repair it.

Use spacing `h=0.5/1535`, boundary width `w=16h`, normalized boundary position
`s in [0,1]`, and quintic smootherstep
`b(s)=6s^5-15s^4+10s^3`. Define one real multiplier:

- for `0 <= f < w`, `p(f)=r(0)+b(f/w)*(r(f)-r(0))`
- for `w <= f <= 0.5-w`, `p(f)=r(f)`
- for `0.5-w < f <= 0.5`, let `s=(0.5-f)/w` and use
  `p(f)=r(0.5)+b(s)*(r(f)-r(0.5))`

Multiply every channel by the same `p(f)`. Do not change support, phase,
delay, channel allocation, completion width, or the raw filter definitions.
Hash the raw bank and the scalar multiplier separately. Do not add per-channel
gains, a second correction pass, analytic derivative estimates, fitted slopes,
or another taper. Quintic endpoint blending is a Signal design inference: the
published frame constructions justify verified frame bounds and canonical-dual
reconstruction, not this specific normalizer.

The multiplier equals the exact tightener outside the two boundary spans. At
DC and Nyquist it retains the exact endpoint scale while its first and second
one-sided derivatives are zero. Even real-output mirroring therefore has no
normalizer cusp at either endpoint. Common multiplication also preserves every
raw filter's frequency support and relative channel geometry. These properties
do not prove a bounded time atom; measured guards remain authoritative.

Batch 29.6R first repeats the Rule 26 reconstruction proof and requires
condition ratio at most `1.25`, dual residual at most `1e-8`, all identity
errors within their existing bounds, finite values, raw-bank hash parity, a
stable nonzero multiplier hash, and exact repeat evidence. Failure stops.
Only reconstruction passage opens the representative channels `0`, `15`,
`16`, `768`, `1534`, and `1535` at the unchanged `1e-12` excluded-energy and
`16384`-frame cap. All-channel guard, derivative and projected-field reproof,
coefficient assembly, and every synthesis surface retain their Rule 26 order.

Batch 29.6R is rejected at reconstruction conditioning. The endpoint-even
candidate has frame minimum `0.4649443041`, maximum `1.4034634949`, and
condition ratio `3.0185626163`, above `1.25`. Exact identity still passes:
dual residual is `7.899949e-11`, peak error `7.275958e-12`, RMS error
`1.992566e-13`, head error `7.048638e-13`, and tail error `0`. Raw-bank hash
`c1014f5fc308c290` and multiplier hash `fd32b38fb8e92972` repeat in the release
proof. The raw hash matches the unmodified Rule 26 bank in the same build.

The representative guard did not run. Endpoint smoothness alone does not
control the complete alias-block frame operator. Do not alter the blend,
boundary width, endpoint values, or channel gains. Before another candidate,
Batch 29.6S must freeze a report-only attribution of the limiting residue
blocks, eigenvalue extrema, boundary-bin ownership, and channel contributions
for the raw, exact-pointwise, and endpoint-even banks. No new preconditioner is
authorized by this result.

### Rule 26B: attribute complete alias-block conditioning before redesign

Batch 29.6T measures one fixed matrix on the unchanged `4096`-frame mixed
control padded to `4224` frames. Use hop `384`, all `11` alias residues, and
exactly three banks derived from the Rule 26 raw boundary bank:

1. raw: no scalar normalization
2. exact-pointwise: multiply every channel by `r(f)=1/sqrt(E(f))` at every bin
3. endpoint-even: the rejected Rule 26A multiplier

The exact-pointwise bank is a diagnostic counterfactual, not a synthesis
candidate. Do not change the raw filters, completion, width, delays, cutoff,
normalizer formula, or eigenvalue estimator between banks.

For every bank and residue, build the same complete Hermitian alias-block frame
matrix used by reconstruction. Report residue index, member-bin count and hash,
minimum and maximum eigenvalue, condition ratio, normalized eigenpair residual,
and stable matrix/eigenvector hashes. Use deterministic phase normalization for
eigenvectors: rotate the largest-magnitude entry to nonnegative real, breaking
magnitude ties by lowest bin index. Require residual at most `1e-6`; otherwise
the attribution is inconclusive and stops.

For each bank's global minimum and maximum mode, report:

- residue, eigenvalue, Rayleigh quotient under all three banks, and eigenvector
  norm mass in DC (`f<w`), interior, and Nyquist (`f>0.5-w`) bins
- the `16` largest bin weights, ordered by weight then bin index, with frequency
  and region; aggregate the remainder
- per-channel quadratic contribution
  `q_k=|sum_i conj(H_k[i])*v[i]|^2`, diagonal part
  `d_k=sum_i |H_k[i]|^2*|v[i]|^2`, and signed cross part `q_k-d_k`
- the `16` largest channels by `q_k` and the `16` largest by absolute cross
  part, ordered by contribution then channel index; aggregate every remainder
- sums of `q_k`, `d_k`, and cross parts, with `sum_k q_k` matching the mode
  eigenvalue within `1e-8` relative error

Report raw-bank, exact-multiplier, endpoint-multiplier, matrix, evidence, and
repeat hashes. All counts and floating-point values must be finite and repeat
exactly within one build profile. Do not reconstruct samples, form canonical
duals, measure atoms, assemble coefficients, run phase logic, or render audio.

The outcome chooses only a research direction:

- if the exact-pointwise bank exceeds condition ratio `1.25`, return to
  boundary geometry; scalar conditioning is insufficient before smoothness
- if exact-pointwise passes, but either endpoint-even limiting mode has less
  than `90%` norm mass in the two boundary spans, return to boundary geometry;
  the smoothness trade is not localized enough for a boundary preconditioner
- if exact-pointwise passes, both endpoint-even limiting modes have at least
  `90%` boundary-span mass, and every eigenpair/contribution gate passes, a
  separately frozen block-aware boundary preconditioner may be researched

No branch authorizes implementation directly. Contract and roadmap work must
freeze the selected next candidate or geometry reassessment first.

Batch 29.6T is numerically inconclusive. All `33` residue rows and six global
mode attributions repeat, and contribution closure reaches `6.650463e-16`, but
the fixed estimator's worst normalized eigenpair residual is `0.031864856`
against `1e-6`. Clustered non-limiting residue modes do not converge enough to
support the direction decision. Do not use the apparent exact-pointwise
condition or boundary mass until every residue has an accurate eigenpair.

Batch 29.6U must freeze one deterministic Hermitian eigensolver proof for these
bounded alias blocks. It must retain the same three banks and matrix hashes,
prove all extremal residuals at most `1e-6`, and cross-check trace and Frobenius
invariants before rerunning attribution. Do not increase power iterations,
relax the residual, or authorize a preconditioner from partial rows.

### Rule 26C: use one cyclic complex-Hermitian Jacobi eigensolver

Batch 29.6V implements one report-only full eigendecomposition for matrices of
size `1..=193`. Reject non-finite input or relative Hermitian error above
`1e-12`; do not symmetrize or repair the matrix. Initialize the eigenvector
matrix to identity and run cyclic Jacobi sweeps over pairs `(p,q)` in
lexicographic order.

For each nonzero upper-triangle entry, apply its unit complex phase to reduce
the `p,q` pivot to a real symmetric `2x2` problem, then use the stable Jacobi
rotation with `tau=(a_qq-a_pp)/(2*|a_pq|)`,
`t=sign(tau)/(abs(tau)+sqrt(1+tau^2))` (`t=1` when `tau=0`),
`c=1/sqrt(1+t^2)`, and `s=t*c`. Update both matrix triangles and accumulated
eigenvectors from the same rotation. Force only the annihilated `p,q` pair to
exact zero; do not zero other small entries.

After each complete sweep, compute off-diagonal Frobenius norm. Converge when
it is at most `1e-13` times total Frobenius norm, with both norms measured from
the current matrix. Stop and reject after `64` sweeps. Do not increase the cap,
switch pivot strategy, relax tolerance, or fall back to power iteration.

Sort eigenpairs by ascending eigenvalue, breaking exact-value ties by the
pre-sort Jacobi column index. Normalize each vector to unit norm, then rotate
its largest-magnitude entry to nonnegative real, breaking magnitude ties by
lowest row index. Report sweep/rotation counts, convergence, input Hermitian
error, final off-diagonal ratio, eigenvalue/eigenvector hashes, and:

- maximum normalized eigenpair residual at most `1e-8`
- maximum orthogonality error at most `1e-10`
- relative trace mismatch at most `1e-12`
- relative Frobenius/eigenvalue-square mismatch at most `1e-10`
- finite values and exact repeat evidence within one build profile

Prove analytic `1x1`, real and complex `2x2`, diagonal, repeated-eigenvalue,
and tightly clustered controls before running all `33` frozen alias matrices.
For actual matrices, eigenvalue extrema must agree with the previous estimator
within `5e-4` relative where that estimator's residual was at most `1e-6`;
unconverged historical rows are not comparison truth.

Passing Batch 29.6V reopens only the unchanged Batch 29.6T attribution with the
Jacobi eigenpairs. Failure returns to numerical-method research. No filter,
dual, guard, phase, coefficient, synthesis, corpus, or product work opens.

Batch 29.6V passes all six analytic controls and all `33` frozen alias
matrices. Release-profile maxima are eigenpair residual `9.186641e-13`,
orthogonality error `9.523849e-15`, trace mismatch `8.882996e-16`, and
Frobenius mismatch `1.344433e-14`. Evidence hash `ac00e9f757b44e7a` repeats.
This reopens only Batch 29.6W: rerun the unchanged Rule 26B attribution with
Jacobi eigenpairs. Do not combine solver proof and direction selection.

Batch 29.6W selects boundary-geometry reassessment. The exact-pointwise bank
has condition ratio `2.9916436058`, above `1.25`, so common scalar conditioning
is insufficient before smoothness. Endpoint-even minimum and maximum modes are
Nyquist-localized with boundary-span masses `0.9972172436` and `0.9973869346`,
but Rule 26B's first branch governs. Maximum eigenpair residual is
`9.186641e-13`, contribution closure is `4.268183e-15`, and evidence hash
`069142f1ee68f2a4` repeats.

Do not research block-aware preconditioning or modify a scalar normalizer.
Batch 29.6X must freeze one boundary-geometry reassessment using this
attribution before any new filter bank is implemented.

### Rule 26D: isolate Nyquist-completion alias coupling before filter design

Batch 29.6Y asks one report-only question: is channel `1535` cross-bin coupling
sufficient to cause the exact-pointwise bank's condition failure? Use only its
already-built `33` Hermitian matrices and proven Jacobi solver.

On every residue, compare exactly three operators:

1. full exact-pointwise frame matrix
2. channel-`1535` removed: subtract its complete rank-one outer product
3. channel-`1535` diagonalized: subtract only its off-diagonal outer-product
   terms while retaining its per-bin diagonal energy

The last two are matrix ablations, not realizable filters or synthesis
candidates. Do not alter responses, normalization, completion width, delays,
hop, bin membership, or any channel other than the stated subtraction.

Report all per-residue extrema and condition ratios, global extrema, Jacobi
proof errors and hashes, channel-`1535` diagonal energy, off-diagonal Frobenius
energy, and the eigenvalue/Rayleigh changes for the frozen exact-pointwise
minimum and maximum modes. Require the Rule 26C numerical gates, finite values,
contribution closure `1e-8`, and exact repeat in release.

The outcome chooses one geometry research boundary:

- diagonalized channel `1535` condition at most `1.25`: freeze separately
  researched orthogonal or multi-row Nyquist completion; do not implement it
- diagonalized condition above `1.25`, but removal condition at most `1.25`:
  freeze a replacement completion family because diagonal energy is also wrong
- removal condition above `1.25`: broaden reassessment to the complete high-edge
  channel geometry; channel `1535` alone is insufficient
- any numerical failure: stop as inconclusive

No result authorizes filter implementation, duals, guards, phase, or synthesis.

### Rule 27: synthesize a protected centre, not a circular endpoint

Extend the source in both directions with whole-sample even reflection,
including the endpoint sample: `x[-1]=x[0]` and `x[N]=x[N-1]`. Analyze the
guarded source through the unchanged tightened filters and auxiliary derivative
filters. Project logical output columns from `-guard_columns` through
`ceil(target_frames/384)+guard_columns`, inclusive, with
`u=m/ratio`. The source coordinate remains relative to the unpadded source;
padding changes storage, not mapping.

Reuse Batch 29.6M interpolation and heap rules on this guarded grid. Assemble
each positive-channel coefficient as projected magnitude times the solved unit
phasor. Transform coefficient rows back to their alias residues, apply the
complete canonical-dual block solve from Batch 29.6J, mirror positive
frequencies, and force DC and Nyquist imaginary parts to zero. After the real
inverse transform, crop exactly `target_frames` from the guard-protected centre.
Do not normalize, fade, zero-fill, or endpoint-correct the crop after synthesis.

Report analyzed/projected/synthesized coefficient counts, guard coverage,
canonical-dual residual, conjugate-symmetry error, maximum imaginary residue,
crop start/end, exact output length, head/tail error, non-finite coefficients
and samples, source/output/coefficient hashes, and repeat hashes.

### Rule 28: synthetic audio gates precede the mono corpus

Run identity, `0.75x`, and `1.5x` on the unchanged steady low/mid/high tone,
two-tone, linear/exponential chirp, broadband impulse, deterministic noise,
mixed tonal/transient, and silence controls. Require:

- selected guard at most `16384` frames and dual-atom tail energy at most
  `1e-12`
- canonical-dual residual at most `1e-8`
- conjugate-symmetry and imaginary-output residue at most `1e-9`
- exact target length with no uncovered, zero-filled, post-faded, or non-finite
  output
- identity peak error at most `1e-5`, RMS error at most `1e-6`, and head/tail
  error at most `1e-5`
- steady-tone angular-frequency error at most `1e-6` radians/sample
- impulse peak within one sample frame of `round(source_peak*ratio)`
- silence peak at most `1e-12`
- identical evidence, coefficient, sample, and trace hashes on repeat

Failure stops before the corpus and returns to the failing guard, assembly,
symmetry, crop, or placement rule. Passing opens only the unchanged fixed-ratio
60-row mono gate. Linked stereo, dynamic ratio, production selection, cache
identity, and product routing remain closed.

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Implement Batch 29.6Y Nyquist-completion matrix ablation and stop after its
geometry research decision. Do not implement filters, duals, or guards.
