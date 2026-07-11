# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; auxiliary derivative-filter estimator passed
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

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Freeze the fractional source-projection and bounded deterministic heap proof
before implementation. Keep synthesis, the corpus, linked stereo, and all
product routing closed.
