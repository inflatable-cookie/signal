# Offline Time-Stretch Synthesis

Status: behavioural-forensics reset active
Owner: dsp
Updated: 2026-07-11
Contract refs: `046`, `082`
Roadmap ref: `g10.029`

## Current Boundary

The production OfflineHighQuality prototype remains the current `2048/512`
identity-lock/reset phase vocoder. The rejected report-only hybrid renders
independent short, current, and long STFT outputs. It is evidence, not the
successor architecture.

## Successor Shape

The next successor candidate owns one sample-domain time map and one
frequency-adaptive, perfectly reconstructable filter bank.

- a painless frequency-adaptive nonstationary Gabor transform gives long
  low-frequency atoms and short high-frequency atoms inside one invertible
  representation
- absolute analysis centres follow the exact requested source map; adjacent
  integer analysis intervals may differ by one frame
- canonical dual filters reconstruct the unmodified coefficients before any
  stretch processing opens
- a later proof may integrate a filter-bank phase gradient, but only after
  reconstruction, impulse delay, coefficient coverage, and determinism pass
- a later stretched candidate must assign every significant coefficient phase
  exactly once from the strongest available time or frequency predecessor
- no peak tracker, onset detector, reset schedule, component mask, independent
  component synthesis, or local timing compensation enters the first proof
- boundary padding, normalized overlap-add, exact cropping, and identity bypass
  remain common synthesis policy

## State Ownership

One fixed-ratio mono engine owns:

- source and output frame cursors
- absolute ideal and rounded analysis-frame positions
- one immutable frequency lattice, analysis-filter bank, dual-filter bank, and
  global ratio
- bounded per-band coefficient storage and exact coefficient time positions
- later phase-derivative and integration state only after the transform proof
- deterministic insignificant-bin phase
- exact output-length and crop state

Linked stereo later shares the time map and phase-propagation decisions.
Channels retain their own complex spectra and phase gradients. Independent
per-channel heap topology is not an acceptable stereo path.

Dynamic ratio remains outside the successor until fixed-ratio mono and linked
stereo pass. Its eventual path must update the same time map continuously; it
must not concatenate independent renders.

## Staged Proof

1. current-grid adaptive transient timeline — rejected after timing and
   combined-gate failure
2. fixed-map peak transient proof — rejected after crest, placement, spectrum,
   and combined-gate failure
3. iterative H/R/P separation and exact reconstruction proof
4. additive H/R/P fixed-ratio mono candidate
5. fixed-resolution full phase-gradient kernel proof
6. whole-band full phase-gradient fixed-ratio mono gate
7. exact-lattice whole-band phase-gradient mono gate
8. frequency-adaptive painless reconstruction proof
9. frequency-adaptive phase-gradient mono mechanism and corpus gate
10. shared-decision linked stereo
11. concealed listening and dynamic-ratio checkpoint

Each stage stays report-only until the complete gate passes. A mechanism proof
may authorize the next stage but cannot promote product quality alone.

## Rejected Shapes

- independently rendered STFT branches joined by waveform crossfade
- bounded delay alignment between those branches
- global removal of identity locking
- scalar phase-lock or long-window selector sweeps
- fixed tail envelopes or hidden output padding
- local unity-ratio attack islands with steady-interval compensation
- two-way H/P processing that forces ambiguous content into a specialized path
- additive H/R/P TSM after its complete mono gate failed
- WSOLA as the next full-band path
- onset-adaptive windows coupled to local unity-ratio attack islands

## Separation Boundary

Contract `082` freezes a refined H/R/P decomposition. Long-resolution analysis
extracts only clearly harmonic bins. Short-resolution analysis extracts only
clearly percussive bins from the complement. The residual owns everything
ambiguous. Binary complement masks and normalized inverse STFT must prove exact
source reconstruction before component TSM opens.

This boundary is retained as proven historical evidence. Additive component TSM
failed and is not the active successor shape.

## Phase-Gradient Boundary

Contract `082` freezes the first active successor proof. It uses a
`4092`-sample Hann window, `8192`-point FFT, fixed `1024`-sample synthesis hop,
and nearest-integer analysis hop derived from the ratio. Centered finite
differences estimate both components of the analyzed phase gradient. A bounded
max heap integrates phase with the published trapezoidal rules.

The implementation operates on the nonredundant spectrum and mirrors synthesis
coefficients to enforce real output. The first frame keeps analyzed phase.
Bins below the frame-pair relative tolerance keep analyzed phase instead of
receiving random values. These are deterministic Signal boundary choices, not
claims about the reference implementation.

The first fixed-hop candidate is rejected but leaves useful tonal evidence. Its
constant rounded analysis hop made the internal ratio differ from the requested
ratio and allowed cumulative source-map drift. The next candidate replaces
that repeated hop with absolute rounded analysis centres. All other
phase-gradient policy remains frozen.

Exact lattice also failed the complete gate. Fixed-resolution phase-gradient
processing is retained as evidence, not the active product candidate.

## Frequency-Adaptive Boundary

The next proof changes transform resolution without changing local time. Use a
frequency-adaptive painless nonstationary Gabor frame: constant-Q-spaced
interior bands, explicit DC and Nyquist completion bands, compact frequency
supports, and canonical dual filters derived from the frame operator. Every
coefficient remains part of one transform and every sample follows the same
global map.

This is not the published onset-adaptive NSG time-stretch algorithm. That
algorithm couples short attack windows to onset detection and unity stretch at
attacks. Signal does not adopt those policies. Batch 29.6I proves only analysis
and synthesis identity, bounded filter support, complete spectral coverage,
band impulse delay, finite coefficients, and repeatability. No phase
propagation or stretched audio belongs in that batch.

Batch 29.6I's unequal per-band decimations do not form one aligned coefficient
matrix. Published filter-bank PGHI covers controlled frequency variation only
on a common time lattice and names nonuniform-decimation integration as future
work. Direct propagation on the Batch 29.6I arrays is closed.

The next proof uses grid-decimated analytic wavelets. Uniform decimation and
deterministic channel delays provide one rectangular coefficient matrix while
wavelet bandwidth still grows with frequency. Canonical dual synthesis must be
proven from the complete uniform-filter-bank frame operator; the painless
diagonal shortcut from Batch 29.6I does not apply.

Batch 29.6J passes that proof. The analysis bank applies deterministic
frequency-response tightening before the complete alias-block frame operator
and canonical-dual solve. This keeps the frozen wavelet centres, bandwidth
progression, channel delays, hop, and redundancy while improving numerical
conditioning. It does not define modified coefficient phase.

## Common-Grid Phase Boundary

Keep both source and output coefficient fields on the proven `384`-frame grid.
For output column `m`, sample source coefficients at the exact fractional
coordinate `u=m/ratio`; do not move synthesis atoms or round a repeated hop.
Interpolate magnitudes and phase derivatives, not wrapped complex samples.

Each channel atom is centred at `n*384+d_k`. Estimate horizontal instantaneous
frequency first, then transport analyzed phase back to the nominal column time
with that frequency and the known deterministic delay `d_k`. Only these
delay-compensated phases may form vertical differences or heap neighbors.
Positive-frequency integration remains authoritative; canonical-dual synthesis
mirrors the solved spectrum for real output.

The first horizontal phase-difference estimator is rejected. At high
frequencies, wavelet bandwidth exceeds the `+/-62.5 Hz` residual interval
allowed by the `384`-frame hop, so heterodyned phase differences alias before
delay compensation. The transform remains valid; phase transport requires an
alias-free estimator such as an auxiliary derivative-filter ratio.

The passing proof derives that auxiliary filter from each finalized, tightened
analysis response. Multiplication by signed angular frequency in the frequency
domain represents the time derivative. At the same coefficient location, the
imaginary derivative/original cross-ratio estimates absolute instantaneous
frequency without inter-column phase unwrap. One maximum-energy qualified
channel supplies the coherent carrier for each column; channel-delay
compensation stays downstream of that estimate. All four tone controls through
`19.5 kHz` pass with maximum error `3.614443e-12` radians/sample and maximum
compensated residual `8.683081e-10` radians.

The next mechanism projects magnitude, absolute instantaneous frequency, and
delay-compensated vertical phase derivatives at exact source coordinate
`u=m/ratio`. Phase integration stays column-local: one magnitude-prioritized
heap carries horizontal candidates from the preceding solved output column and
vertical candidates from solved adjacent channels. Its fixed `3072`-entry cap
does not grow with render duration. Wrapped complex coefficients are never
interpolated. The mechanism passes all `30` synthetic control/ratio cases with
zero coordinate error, no duplicate or missing assignment, and maximum heap
occupancy `1756/3072`.

Audio synthesis cannot crop from a zero-origin circular transform without
allowing the terminal seam to contaminate the head. Before coefficient
assembly, measure the finalized canonical-dual atoms and select the smallest
whole-hop two-sided guard whose excluded energy is at most `1e-12` for every
channel, capped at `16384` frames. Reflect the source at both endpoints,
synthesize the guarded coefficient grid, then crop the protected centre. No
post-synthesis fade, normalization, zero fill, or endpoint correction may hide
a boundary failure. The exact guard proof rejects the current bank before
assembly: lowpass channel `0` retains `6.270779e-7` excluded energy at the
largest legal support radius and requires more than `16384` guard frames. The
complete-frame solve itself remains accurate at `1.051210e-12` residual.

Tail attribution now compares raw analysis, tightened analysis, and exact
canonical-dual atoms at fixed radii for representative lowpass, interior, and
edge channels. Positive-only analytic atoms and conjugate-mirrored real-output
atoms remain separate measurements. This isolates filter, tightening, dual,
mirroring, and lowpass ownership without changing transform geometry.

The attribution matrix isolates two boundary defects. Per-bin tightening raises
the channel `0` real-output tail from `1.622121e-13` to `6.270779e-7`; exact
dualization is neutral. At Nyquist, channel `1535` already carries
`1.180453e-7` raw tail and reaches `2.030199e-7` after tightening and
dualization. Representative low/interior wavelets are compact at the measured
radius. The next transform work must jointly smooth real-output DC and Nyquist
completion while preserving the interior bank.

The frozen candidate removes global pointwise tightening, retains raw channels
`0..1534`, and replaces only channel `1535` with a zero-delay, endpoint-flat
Nyquist completion across the existing `16`-spacing completion width. It does
not assume a tight partition. Complete frame bounds and the exact canonical
dual remain authoritative. Representative boundary guards precede the expensive
all-channel scan; phase mechanisms reopen only after both reconstruction and
all-channel guard passage.

The candidate passes exact canonical-dual identity but fails its first hard
gate. Frame energy spans `0.7361080721..2.1937926704`, giving condition ratio
`2.9802589505` against the `1.25` cap. The representative guard therefore does
not run. The next design boundary is conditioning: freeze one smooth,
endpoint-compatible preconditioner or normalizer that does not recreate the
DC tail caused by pointwise tightening or change the frozen Nyquist completion.

The frozen preconditioner is one common real multiplier. It uses exact inverse
square-root frame energy in the interior and blends that function to its exact
endpoint values with quintic smootherstep across the existing `16`-spacing DC
and Nyquist spans. The multiplier has zero first and second endpoint
derivatives, preserves raw filter support and relative channel geometry, and
requires no fitted derivative. This removes the known real-mirror cusp by
construction; only measured conditioning and dual-atom guards can establish
that the complete bank is usable.

The endpoint-even candidate fails the complete frame gate. Condition ratio is
`3.0185626163`, with eigenvalue extrema `0.4649443041` and `1.4034634949`,
despite exact identity reconstruction. The common scalar controls pointwise
energy but not the alias-block eigenstructure created by `384`-frame
decimation. No guard runs. The next proof must attribute limiting residue
blocks and channel ownership across the raw, exact-pointwise, and endpoint-even
banks before another preconditioner shape is frozen.

Attribution uses the complete `11`-residue Hermitian frame matrices, not only
pointwise energy. It compares raw, exact-pointwise, and endpoint-even versions
of the same boundary bank. Per-residue extrema identify the limiting blocks;
global limiting eigenvectors are then decomposed by DC/interior/Nyquist bin
mass and by per-channel diagonal and cross-term contribution. Cross-bank
Rayleigh transfer distinguishes a scalar-tightening failure from a localized
endpoint-smoothing failure. The report chooses only between boundary-geometry
reassessment and later block-aware preconditioner research.

The first attribution run is inconclusive: contribution closure passes, but
clustered residue modes leave a worst normalized eigenpair residual of
`0.031864856`. A deterministic bounded Hermitian eigensolver proof must replace
the fixed-start power estimator before the same attribution can choose a
direction.

The frozen numerical replacement is a full lexicographic cyclic
complex-Hermitian Jacobi solve for blocks up to `193`. It fails closed on
Hermitian error, non-convergence after `64` sweeps, residual, orthogonality,
trace, or Frobenius mismatch. Only a passing eigensolver proof may rerun the
existing attribution.

The Jacobi proof passes all analytic controls and `33` alias matrices with
maximum residual `9.186641e-13` and invariant errors below `1.4e-14`. The same
attribution may now replace only its eigenpairs and make the frozen direction
decision.

The accurate attribution selects boundary geometry. Exact pointwise scalar
normalization still has condition ratio `2.9916436058`, while both limiting
endpoint-even modes carry more than `99.7%` of their mass in the Nyquist span.
Scalar and block-aware preconditioner work is closed pending one geometry
reassessment contract.

That reassessment isolates channel `1535` before any filter design. The exact
pointwise extrema both occur in residue `0`; their dominant bins are `2101`
and `2112`. Channel `1535` contributes a cross term near `-0.491` to the
minimum mode and `+0.492` to the maximum mode. The next report compares the
full frame operator with two diagnostic ablations across every residue: remove
the complete channel-`1535` rank-one term, or remove only its off-diagonal terms
while retaining diagonal energy. These are matrix probes, not realizable
filters.

The ablation isolates the defect. Removing only channel `1535` off-diagonal
coupling reduces global condition from `2.9916436058` to `1.1141796230`.
Removing the complete channel instead leaves condition `2.6496906694`, so its
diagonal energy is necessary. The next geometry must distribute or
orthogonalize Nyquist completion while preserving that energy; no realizable
response has been selected.

The frozen realizable geometry replaces the one completion row with three
equal-energy rows at delays `-128`, `0`, and `+128` frames. Their DFT phase
coding cancels alias-bin separations of one and two hops; the retained support
is narrower than three alias intervals, so these are the only possible
off-diagonal pairs. The rows preserve the original summed diagonal energy and
are all real at Nyquist. This is a proof candidate, not a promoted filter bank.

The triplet proves its local mechanism but fails the complete bank. Its
completion cross terms close below `5e-15`, while complete condition remains
`2.0862893665` with limiting residues `3` and `8`. The one-row Nyquist defect
was real but not sufficient ownership of the untightened bank's conditioning.
Another candidate is blocked pending attribution of the remaining boundary
geometry.

The residual attribution now separates the unchanged raw boundary groups
without designing another response. It compares the full triplet candidate
with off-diagonal coupling removed from DC rows `0..15`, preserved high-edge
rows `1520..1534`, or both. Completion rows remain unchanged. Per-mode group
contributions and complete-bank conditions will select DC, high edge, joint
boundary, or broader raw-bank ownership.

The report selects broader raw-bank ownership. DC cross-term removal is
neutral; preserved-high-edge removal raises condition from `2.0862893665` to
`2.1170081614`. The limiting modes remain Nyquist-localized, but endpoint-group
cross coupling does not own the failure. Further endpoint variants are closed
pending a complete-bank reassessment.

The final common-grid feasibility candidate applies the exact canonical
inverse square root to every complete alias block. This guarantees a tight
frame algebraically, but mixes alias-separated bins. The decisive evidence is
therefore compact-support leakage and all-row inverse-FFT localization, not
condition alone. Failure closes the common-grid family rather than opening
another correction.

Canonical tightening reaches numerical identity but fails the frozen support
gate: row `12` expands from `19` bins to all `2113` positive bins. The peak
introduced outside support is `1.2528705611e-12`. This closes common-grid
correction work without claiming audible damage; transform-family reassessment
must now account for localization explicitly.

The reassessment returns to the passing painless nonstationary-Gabor bank, not
the rejected wavelet bank. Keep its filters and diagonal canonical dual, but
place every band on one dense coefficient lattice using the largest existing
per-band coefficient count. This removes unequal-lattice adjacency without an
alias-block solve or support-mixing tightener. It is a new feasibility question,
not a promoted successor: coefficient cost, real-boundary closure, identity
reconstruction, and large-probe atom localization must pass before phase work.

That dense proof is rejected. At `65536` frames it requires hop `4`, redundancy
`208`, and `72.7455x` the unequal-lattice coefficient count. Identity
reconstruction and condition pass, but real-spectrum closure is
`1.7881393433e-7` and the worst analysis and dual atoms retain
`0.4999847412` excluded energy at radius `16384`. No phase topology opens.

Operator direction keeps successor research open and changes the adaptation
axis. The next family is one time-adaptive painless nonstationary Gabor frame:
compact time windows become short in declared transient regions and remain long
in stationary tonal regions. Every frame retains one `4096`-bin spectrum and
one global source map. A diagonal time-domain frame operator supplies the exact
dual; no independently synthesized resolution branch exists.

The first proof uses declared schedules only. Automatic selection, onset or
percussion detection, phase modification, and stretched synthesis remain
closed until identity reconstruction, coverage, conditioning, compact support,
real output, and deterministic schedule transitions pass.

That identity proof passes all five schedules and eleven controls. Adaptive
schedule condition is at most `1.5934675721`; worst peak reconstruction error
is `7.2164496601e-16`, conjugate-symmetry error `4.8233240331e-13`, and
imaginary residue `3.4192121536e-16`. Automatic resolution selection is now the
only open research question. Phase and stretched synthesis remain closed.

Automatic selection uses one evidence family: local Rényi entropy across the
four passing window resolutions. Every candidate is measured over the same
source region with lattice-area normalization. One bounded offline path chooses
minimum total entropy while allowing adjacent resolution levels to differ by at
most one; exact ties prefer longer windows. Stereo later sums channel energies
before normalization and shares the resulting path. No onset, HPSS, flux, peak,
or learned classifier participates.

The frozen selector is rejected. It preserves long windows on steady tones and
passes invariance/stability gates, but one impulse selects `512` across `36/64`
anchors, the linear chirp selects `512` everywhere, and mixed tonal/transient
audio selects `4096` everywhere. The fixed comparison region spreads isolated
event ownership while whole-band tonal energy hides the mixed transient. Phase
work remains closed pending selector-failure attribution.

Selector-failure attribution now has a fixed report-only boundary. The exact
Batch 29.6AK coefficient field is partitioned two ways: eight time slices by
frame centre and eight folded-frequency regions. Both views must close the
unchanged energy and Rényi sums. Leave-one-region-out entropy is diagnostic
only: it may identify comparison-region geometry, frequency evidence, or an
inconclusive split, but cannot alter a path or schedule.

That attribution is inconclusive. Event-facing time slices explain only part
of the isolated-event spread and also disturb mixed stationary controls. The
lowest folded-frequency region explains every mixed event miss and much of the
linear chirp behavior, but also changes one stationary mixed anchor. The coarse
time-centre and frequency partitions therefore expose coupled mechanisms rather
than one clean selector boundary.

One final attribution refinement is bounded before operator review. Window
support membership replaces centre-only time slices, testing exactly which
coefficient frames can contain the declared event. Only the implicated
`0–3 kHz` folded region is subdivided, into eight fixed roughly `375 Hz`
regions. These remain diagnostic removals over unchanged coefficients. Passage
may identify geometry, frequency evidence, or a joint localized time-frequency
boundary; it cannot change selection.

The refinement selects comparison-region geometry. Removing frames whose
analysis support can contain the declared event fixes every distant isolated
decision without disturbing stationary mixed controls. Narrow low-band removal
is not selective: the only event-restoring subregion changes every negative
control. Event labels remain proof fixtures, not selector input; the next
contract must express one source-blind Rényi comparison geometry.

The selected geometry is anchor-local and support-contained. Each resolution
uses its natural-hop centres symmetric around the decision anchor, but only
when the complete analysis window fits inside the unchanged `4096`-frame
comparison region. Counts are `[29,13,5,1]`. FFT work may be shared across
anchors without changing membership. This removes outside-region support
leakage while retaining one Rényi evidence family and one legal path.

The terminal geometry is structurally correct but musically rejected. The
isolated-event legal path retains transition shoulders outside the frozen
far-field boundary, mixed tonal/transient audio remains all-long, and impulse
controls exceed the perturbation cap. Automatic Rényi selector research is now
stopped for operator review. Transform reconstruction remains valid; only
automatic schedule selection is blocked.

Operator direction retires Rényi-only automatic selection and opens one
transient-aware evidence family. The candidate is magnitude-gated mixed phase
derivative occupancy: a pre-analysis mask identifies impulsive time-frequency
bins and reduces them to one per-frame percussive ratio. This is analysis only.
No harmonic/percussive component audio is separated or resynthesized, and the
paper's adaptive hop, phase, and empirical threshold policy does not transfer.

The first detector is now fixed. A `2048`-frame pre-analysis measures centered
mixed phase increments on the `128`-frame grid and normalizes the ideal
sinusoidal/impulsive values to `0/1`. A scale-relative numerical energy floor
removes unstable cells. Linked percussive magnitude divided by linked eligible
magnitude yields one occupancy ratio. Midpoint classification and a `0.5` local
peak rule are analytic; no smoothing or detector vote is present.

That analytic detector is rejected. It fires on every steady/chirp/noise
negative family, localizes isolated and boundary impulses poorly, cannot resolve
the dense pair, and is highly perturbation-sensitive. The mixed center event is
visible but accompanied by outer-quarter false positives. Transient-aware
selection returns to operator review before any calibration or schedule mapping.

Operator direction keeps the mixed-phase family but does not authorize blind
threshold fitting. The primary method's absolute magnitude threshold is not
scale-invariant, its median-filter length is unspecified, and its empirical
phase interval must be reconciled with Signal's normalization. The next proof
therefore measures normalized-magnitude and mixed-phase distributions for event
and negative controls. Only demonstrated separation may open a later calibrated
mask contract.

That distribution audit rejects calibration. Every fixed cutoff/radius pair
overlaps; chirp magnitude remains strongly concentrated near the nominal
impulsive mixed-phase value, while the mixed event loses most of its evidence
under useful magnitude cutoffs. Boundary equal-energy stereo also crosses a
cutoff boundary. The mixed-phase family is stopped before smoothing,
prominence, schedule mapping, or audio.

Operator direction selects median HPSS as the next evidence family. One linked
magnitude spectrogram is filtered vertically over 17 bins for percussive
structure and horizontally over 149 decision frames for harmonic structure.
The time filter preserves the physical duration of FitzGerald's 17-frame,
1024-hop example on Signal's 128-hop grid. A `p=2` soft mask reduces the two
estimates to percussive occupancy. No separated waveform is synthesized.

Median HPSS is numerically and stereo stable but rejected as an event detector.
Every negative family peaks; impulse, boundary, dense, and mixed event placement
fails; impulse perturbations are unstable. Mixed phase and median HPSS therefore
fail despite different cell evidence. The common failed abstraction is reducing
percussive occupancy to local event peaks. Selector work returns to operator
review before another evidence family or schedule mapping.

The strategy checkpoint stops that loop. Automatic selection is now downstream
of an oracle value proof. One end-to-end candidate uses manifest-declared event
intervals to drive the passing four-window painless schedule, maps absolute
source centres to fixed-ratio output centres, generalizes the current
identity-locked phase policy to actual variable hops, and synthesizes through
the exact output-side diagonal dual. This tests adaptive magnitude resolution
without detector error, component branches, phase resets, or local time warps.

The oracle candidate fails before that real-source gate. Identity, schedule,
mapping, coverage, numerical, and repeat evidence pass, but its `1.5x` isolated
impulse lands `127` frames early. The 15-row corpus and concealed comparison do
not run. The time-adaptive successor and automatic-selector work are retired.

That rejection does not close Signal-native quality work. It exposes an
architecture error in the research program: Signal prohibited local timing,
transient phase reset, joint mechanism tuning, and simultaneous
multi-resolution synthesis even though mature stretchers use those mechanisms
as an interacting system.

The active architecture is therefore behavioural forensics before synthesis.
Rubber Band R2 and R3 remain external specimens, not dependencies. Generated
controls and the existing licensed corpus measure event-local time maps,
transient treatment, vertical phase behaviour, and R3 standard-versus-short
resolution deltas. Public offline introspection may supply output increments,
phase-reset curves, and exact-time points where the installed comparator
supports them.

Signal may reopen a nonuniform local time map and coordinated transient phase
treatment. Exact requested duration remains mandatory, but exact local ratio is
not. A future multi-resolution candidate must combine resolutions inside one
synthesis system; selecting one window per frame or crossfading independent
renders is not assumed sufficient.

## Next Task

Run the Rubber Band behavioural-forensics contract. Freeze probes and report
schema before implementation. Keep a new Signal synthesis candidate, product
promotion, and routing closed until the measured mechanism signatures select a
complete architecture.
