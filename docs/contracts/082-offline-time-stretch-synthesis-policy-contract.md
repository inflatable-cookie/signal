# 082 Offline Time-Stretch Synthesis Policy Contract

Status: active; frequency-adaptive reconstruction proof passed
Owner: dsp
Updated: 2026-07-10
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

## Clean-Room Rule

Public papers and public algorithm descriptions may inform Signal design.
Rubber Band source, unpublished R3 behavior, Elastique internals, and copied
implementation details are outside the research and implementation boundary.

## Next Task

Research and contract the frequency-adaptive phase-gradient mechanism. Define
coefficient-time mapping, derivative scaling, cross-band adjacency, symmetry,
and a synthetic stop gate before implementation. Keep corpus rendering, linked
stereo, and all product routing closed.
