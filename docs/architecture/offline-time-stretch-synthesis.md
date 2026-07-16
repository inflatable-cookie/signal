# Offline Time-Stretch Synthesis

Status: coherent fixed-grid mono baseline validated; linked stereo source study
active after calibrated repair rejection
Owner: dsp
Updated: 2026-07-16
Contract refs: `046`, `082`
Roadmap ref: `g10.029`

## Current Boundary

The production OfflineHighQuality prototype remains the current `2048/512`
identity-lock/reset phase vocoder. All successor work is report-only. The
source-studied coherent fixed-grid predictor has completed its exact-source
mono comparison against Rubber Band R3 with a material-dependent split and no
overall winner. It may advance to linked-stereo proof. No successor has earned
product routing or dynamic-ratio work.

## Successor Shape

The successor owns one sample-domain time map and one coherent fixed-grid
weighted predictor.

- a `30 ms` output interval and fourfold support define the schedule
- a periodic Kaiser window and modified half-bin transform form one inseparable
  analysis representation
- preliminary horizontal transport precedes ascending short/long vertical
  re-prediction from both frequency directions
- target-energy normalization follows combined prediction, with current-input
  fallback for weak evidence
- exact length, one output timeline, conjugate symmetry, identity behavior,
  and deterministic output remain common synthesis policy

## State Ownership

One fixed-ratio mono engine owns:

- source and output frame cursors plus rounded projected input centres
- the current and auxiliary analysis spectra
- prior preliminary and corrected output state
- short/long lower/upper prediction evidence and weak-evidence fallback
- overlap normalization, exact output length, and crop state

Linked stereo later shares the time map and phase-propagation decisions.
Channels retain their own native complex coefficients and interchannel phase.
Independent per-channel event, owner, or frame-selection decisions are not an
acceptable stereo path.

For the coherent fixed-grid predictor, the shared decision surface is narrower
and explicit. Both channels share ratio projection, frame centres, transform
geometry, frequency traversal, neighbour availability, and one aggregate
correct-versus-fallback mode per frame/bin. Each channel retains its current
and auxiliary spectra, previous input energy, preliminary and corrected phase
recurrence, target magnitude, output accumulation, and normalization. Aggregate
mode energy is the sum of channel energies under the mono predictor's existing
energy-relative floor. No mid/side resynthesis, dominant-channel phase
replacement, cross-channel sample mixing, or independent channel schedule is
allowed.

Inside an aggregate corrected bin, each channel normalizes its own prediction
to its own target energy. An exactly silent target stays zero. If either
significant channel has individually degenerate prediction, both channels take
the shared fallback mode. This keeps one audible mode decision across channels
without fabricating energy or discarding per-channel phase evidence.

Shared scheduling and mode selection are necessary but not sufficient.
Reference-relative recurrence now selects one per-bin channel prediction and
derives its peer from the current input relation. It restores broadband delay
and sharply reduces the original phase/image collapse, but calibrated evidence
still finds material tone and correlated-image drift against ideal and Rubber
Band behavior.

Coefficient, real-edge, overlap, normalization, initial-frame, fallback, and
weak-bin attribution do not own the residual. A render-wide real `2x2` Gram
color transform is also rejected: it closes aggregate covariance but fails
tone IPD and interior-image gates and is not consistently better across local
windows. Stereo repair therefore remains inside linked analysis/phase/synthesis
decisions. Post-render image correction is not an acceptable substitute.

Dynamic ratio remains outside the successor until fixed-ratio mono and linked
stereo pass. Its eventual path must update the same time map continuously; it
must not concatenate independent renders.

## Staged Proof

1. native-grid active-owner mechanism and complete synthetic gate
2. frozen nine-row mono development gate
3. mono decision checkpoint and concealed listening only if earned
4. shared-decision linked stereo
5. cross-channel recurrence research if the linked quality gate fails
6. holdout and dynamic-ratio checkpoint

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

The synthetic comparator matrix confirms those mechanisms interact. Exact
final duration coexists with event displacement in every non-identity mode.
Disabling R2 transient reset changes attack crest and replicas more consistently
than event placement, so transient phase treatment does not own the time map by
itself. Disabling lamination changes phase-coherence and attack-shape evidence
across tonal, event, and mixed families. R3 standard versus short changes event
placement in `23/30`, vertical coherence in `52/56`, and spectral residual in
`49/56` comparable rows. No single direction wins every family.

The architecture therefore requires five cooperating policies: offline study,
bounded local time allocation, event-conditioned phase treatment, simultaneous
multi-resolution synthesis, and linked-channel decisions. Waveform displacement
alone could not define the allocator, so public C++ output-increment and
exact-time-point evidence was required before stage ownership could close.

Direct R2 state now separates those stages. Reset-disabled study produces the
same detector curve as default but selects different exact-time points and a
different output-increment schedule in every measured row. Lamination-disabled
study is identical to default in every row. Signal's architecture must compute
event evidence first, select bounded timing constraints second, construct a
globally exact local schedule third, and apply event and vertical phase policy
as distinct synthesis stages. Rubber Band's signed increment encoding does not
transfer.

The contracted successor is one simultaneous union frame with square-root-Hann
layers at `512`, `2048`, and `8192` frames for its mechanism baseline. Every
layer analyzes the source; none is selected as the sole owner of a time frame.
All atoms synthesize through one exact output-side frame operator and canonical
dual. This preserves a single waveform while retaining short attack and long
tonal evidence concurrently.

Linked offline study emits continuous evidence before application. A separate
policy chooses exact points. A constrained optimizer then produces positive
integer output hops, permits no more than `256` frames of selected-event
movement, favors local unity slope near protected support, and returns all
deviation to zero at the exact target boundary. Event phase correction and
cross-resolution vertical alignment run after ordinary actual-hop transport and
remain separate from the schedule.

The first proof geometry is not a production constant. Complete-system tuning
is capped at `108` configurations over three window banks, two study
sensitivities, three local-unity strengths, three reset scopes, and vertical
alignment on/off. Nine existing
mono rows are development material; six are locked holdout. Objective measures
enforce hard safety and construct a Pareto frontier. Concealed listening selects
at most one candidate. Holdout failure permits no retuning.

Development listening rejects the complete system before holdout. Across the
four explicitly ranked rows, all three successors are unusably blurred while
current Signal and Rubber Band remain tight or usable. The defect sounds like
reverb or multiple source copies separated by very small delays. Five
unranked rows cannot recover the required `6/9`; holdout remains unread.

Cross-resolution attribution confirms the hypothesis. Across `108` frozen
development renders, complete-mode layer arrivals disagree by `172.776515`
frames on average and up to `507`. Pairwise correlation is `0.197448`, while
recombination adds `2.145833` replicas per event over the layer mean. Ordinary
mode is already broken; event reset and one-bin vertical alignment barely move
the result. Exact `3.34e-16` layer-sum closure excludes accumulation error.

Independent full-band phase transport is retired. The next proof keeps the
union magnitudes and canonical dual but owns synthesis phase once on a common
physical-frequency field. Every layer projects analyzed phase and frequency to
the common atom centre. One phase is transported and event-corrected there,
then projected back to each resolution with explicit time/frequency offsets.
If full-field coherence cannot meet the frozen development metrics, the union
must change to non-duplicating coefficient ownership.

The shared-field proof fails. It retains exact structure and reduces combined
replica growth, but mean layer-arrival disagreement remains `162.261364`
frames, correlation falls to `0.134045`, and replica growth remains positive.
The defect is therefore representation ownership, not merely missing phase
lamination.

Redundant full-band union synthesis is closed. The next architecture must give
each coefficient one synthesis owner while retaining simultaneous access to
short and long evidence. The bounded review compares complementary source
subbands, explicit cross-resolution coefficient tiling, and one invertible
adaptive-resolution representation. No synthesis implementation resumes until
one family proves exact unmodified reconstruction, event-local ownership
continuity, one time map, boundaries, and linked-stereo decisions on paper.

The review selects one time-adaptive painless nonstationary Gabor frame.
Complementary source subbands provide clean fixed-frequency ownership but not
event-local resolution without a time-varying transition problem. Quilted
Gabor systems support local coefficient selection and reconstruction in proven
frame cases, but generic exact duals and phase transport do not provide the
bounded local implementation boundary Signal needs. The painless NSG family
has both direct time-stretch precedent and passing Signal reconstruction
evidence.

This is not the rejected oracle system unchanged. Batch 29.6BA proved that a
fixed-ratio mapping without event phase treatment places an isolated `1.5x`
impulse early. The selected architecture combines the already passing
single-frame representation with the later complete-system study, globally
exact local output-hop schedule, and separate event and vertical phase stages.

One legal resolution owns each analysis centre. Adjacent selected windows form
one covering frame and synthesize through its exact diagonal dual. There are no
additive resolution layers, coefficient masks, resolution crossfades, or
per-resolution time maps. Selected event support requests the shortest legal
window; legal adjacent-level transitions move monotonically toward longer
windows outside protected support. This is declared schedule geometry, not an
automatic detector or fitted mask.

All coefficients use the same positive-integer globally exact output-hop
schedule. Ordinary physical-frequency phase transport uses actual output hops.
Resolution changes do not reset phase by themselves. Peak-region vertical
locking occurs inside the selected frame; selected event correction remains a
separate downstream operation. Whole-sample even reflection bounds source
reads, every synthesis frame touching the exact crop participates, and output
length comes from schedule plus crop rather than fill. Linked channels share
study, resolution, mapping, peak, and reset decisions while retaining
per-channel spectra and interchannel phase.

The single-owner mechanics proof passes without changing the earlier identity
system. All five declared schedules have one unique window and coefficient
vector at each centre, no count mismatches, bounded selected-frame work, and
the unchanged `6987080e517f1aec` identity hash. The representation boundary is
closed.

The next proof attaches timing without synthesis phase. Existing linked study
points drive the already proven short-window island geometry. Because every
adaptive centre remains on the `128`-frame base grid, each centre reads the one
globally exact output position already owned by the local schedule. Resolution
does not interpolate, quantize, or own another map.

That attachment now passes. All three frozen ratios produce the same
`53/24/16/11` window-count shape over `104` frames, with `81` in-range and `23`
reflected frames. Every in-range centre reads the exact shared schedule entry;
all ownership, transition, monotonicity, endpoint, linked-order, and mapping
checks are zero. The mapping evidence hash is `3ea1d3a2297083e2`; earlier
identity and ownership evidence is unchanged.

The next proof is the first stretched synthesis for this representation. One
phase state follows the selected coefficient vector across actual source and
output hops; a window-length change is not a reset. Selected-event correction
and peak-region vertical locking remain distinct, current-frame operations.
Before phase quality is interpreted, the moved windows must prove positive
output-lattice coverage and an exact diagonal dual over the protected crop.

That mechanism proof passes. The moved adaptive windows cover every exact crop
with condition at most `2.964471`. One phase state crosses all `24` resolution
changes and initializes once per channel. Identity, coefficient/magnitude
ownership, timing ownership, exact length, symmetry, residue, linked decisions,
and repeat pass. The combined mode preserves the `311 Hz` control within
`0.5 Hz`; known injected attacks remain within `256` frames. Evidence hash is
`9cc7519deb368966`.

This is synthesis liveness, not a quality win. The next gate freezes this
combined mode and measures isolated and dense event timing, crest, replicas,
tonal spectrum and texture, silence, and boundaries on synthetic controls.
Corpus audio remains closed until those absolute checks pass without tuning.

That quality gate rejects the candidate. Exact structure, identity,
coefficient/magnitude ownership, silence, symmetry, residue, finiteness, and
repeat remain intact. Stretched steady tones and scheduled events do not:
`48` control/ratio rows produce `25` hard failures and one combined-mode
regression. Angular-frequency error reaches `6.842e-4` radians/sample;
isolated-event displacement reaches `496` frames; dense one-to-one displacement
reaches `896` frames. The combined mode improves several dense rows but does not
repair isolated placement or general pitch and causes the one regression.

The next stage is trace-only. It freezes the failed rows and follows frame-hop
phase advance and event-local diagonal-dual contributions through to output
energy and peaks. This must locate the earliest responsible boundary before any
phase, event, vertical, or synthesis redesign. Corpus audio remains closed.

That trace assigns the failures before another candidate is built. Ordinary
physical-frequency phase transport owns `14`; event ownership/frame attachment
owns `10`; event correction owns the one combined-only regression. Vertical
locking and diagonal-dual synthesis own none. The tone path changes dominant
ownership `738` times across `2,298` frame records and carries larger frequency
error on same-resolution frames than at resolution changes. The event path
selects none of `18` injected attacks and centres only six coincidentally.

The next mechanism therefore has two separate owners. Active spectral-peak
trajectories carry physical-frequency state; newly active peaks initialize from
current analysis phase instead of dormant bins. Sample-refined transient
anchors are detected independently of resolution points and become exact
source/output frame centres. The existing painless transform, global duration,
and diagonal dual remain fixed.

That ownership proof passes. A fixed analytic tracking spectrum supplies
physical-frequency peak trajectories while the adaptive painless coefficients,
windows, and diagonal dual remain unchanged. Ordered one-to-one peak matching
owns synthesis phase; births initialize from current analysis phase and exact
event-centred frames reset from current analysis at the anchor. Linked
time-domain derivative-energy evidence refines accepted events to sample-frame
positions before the successor schedule attaches them to exact global-map
outputs.

Across `32` mechanism rows, all eight hard failure classes are zero. Rendered
and matched-owner interior tone error stay below `1e-6` radians/sample; all
`24/24` expected anchors detect and attach exactly; identity stays below
`6.674e-16`. Evidence hash `a2d3fb95545cb47f` repeats. A dense-event
rendered-peak diagnostic remains at `262` frames, so mechanism passage does not
claim complete quality passage.

The complete frozen quality matrix confirms that limit. The successor clears
every hard check except `DenseEvent` at `2.0x`: its first dominant peak lands
exactly and its second is `262` frames from target against the `256`-frame cap.
All tone, isolated-event, identity, structure, symmetry, finiteness, silence,
and boundary checks pass with zero regressions. Evidence hash
`c72c005d0cd44e3e` repeats.

Rule 30R shows that the six-frame margin was misleading. Both real `2.0x`
attacks land exactly at outputs `16126` and `16644`, with amplitudes `1.0` and
`0.75`. Overlapping synthesis creates a third `0.787177` peak between them at
`16382`; the unchanged matcher correctly exposes it as a `262`-frame error.
Anchor attachment, event reset, active-owner transport, and exact-sample
contribution closure pass. The defect is an event-local duplicate, not timing
drift or a metric artifact. Evidence hash `2336b9773c32b2ca` repeats.

The overlap owner is now explicit. A non-anchor frame may keep ordinary
background overlap, but when it straddles multiple accepted anchors whose
projected owner supports have separated, each bounded attack neighborhood is
replaced by interpolated boundary background. The exact anchor frames retain
the attacks. On the frozen dense control this removes the sole midpoint replica
while preserving both target samples exactly; passing ratios remain bit-exact.
The unchanged synthetic quality matrix passes all `48` rows. Evidence hashes
`adf37bdd72012e19` and `dec15b718aa27de9` repeat.

The first frozen real-source objective rejects that successor before listening.
All current, successor, and captured-external renders retain exact length,
finite output, and full-render integrity. The successor nevertheless regresses
current Signal on event placement in `6/9` rows, replicas in `7/9`, static
spectral residual in `9/9`, and formant-envelope residual in `9/9`. Its tonal
movement improves in `7/9`, so physical-frequency tracking is doing useful
work, but the complete synthesis does not preserve enough event or spectral
structure. The next proof is a frozen stage ablation, not parameter tuning.

That ablation locates the dominant damage before active tracking or event
ownership. Moving from current Signal to ordinary adaptive synthesis worsens
timing in `8/9` rows, replicas in `7/9`, and both static-spectrum and formant
residuals in `9/9`; seven ordinary renders also breach endpoint-energy limits.
Active-peak transport recovers most mean timing loss and some spectral/formant
loss. Anchors make smaller mixed changes. The `64`-frame overlap owner changes
none of the nine real-source outputs. The next boundary is therefore the
ordinary adaptive representation: fixed window resolution versus resolution
transitions versus their shared phase/output lattice.

Fixed-resolution attribution shows that this boundary has three layers.
Endpoint integrity improves monotonically across the existing window bank:
fixed `512`, `1024`, `2048`, and `4096` fail `9/9`, `9/9`, `4/9`, and `0/9`
rows, while adaptive fails `7/9`. Adaptive synthesis also has the largest mean
timing loss. Resolution and transitions do not explain the timbral defect,
however: every fixed length and adaptive ordinary synthesis regresses both
static-spectrum and formant residual in all nine rows. That common damage now
belongs to the shared phase transport, output lattice, diagonal-dual overlap,
or their interaction.

The fixed-`4096` factor matrix excludes those three mechanisms as the primary
timbral owner. Global-linear placement is nearly neutral when transport and the
exact dual stay active. Analysis-phase passthrough increases static residual in
every row. Replacing the exact dual with an analysis-window partition increases
both static and formant residual in every row. Because all eight combinations
still regress both fields in all nine rows, the next boundary is the shared
windowed coefficient representation. Signal's adaptive renderer analyzes with
a square-root Hann; the current production phase vocoder uses Hann. Analysis
leakage and synthesis weighting must be separated before redesign.

Hann analysis and synthesis both reduce the defect. Hann/Hann roughly halves
mean event timing loss and lowers mean static/formant residual, but every
window pair still regresses both timbral fields in every row. The window kernel
is therefore a contributing mechanism, not the primary owner. The remaining
representation gap is geometric: the successor centers reflected frames and
places shorter windows on a shared `4096` FFT grid, while current Signal uses
start-aligned padded `2048` frames on a native grid. Those factors must be
separated before changing phase or magnitude policy.

Geometry attribution keeps Hann/Hann `2048` fixed and separates those factors.
Moving centered reflected frames from the shared `4096` grid to native `2048`
reduces mean static/formant residual by `0.040495/0.017523` and mean timing loss
by `32.194444` frames, but raises replica ratio by `0.842327`. Replacing
reflection with start-aligned zero padding then worsens static/formant residual
by `0.029572/0.011684`. Every geometry still regresses both timbral fields in
all nine rows. Shared-grid zero-padding contributes to the damage and reflected
boundaries help, but neither owns the broad defect. The remaining owner is the
phase/magnitude coefficient path; native-grid timbral gains cannot be promoted
while replica protection fails.

## Contracted Coefficient Path

Rule 30Z closes the attribution loop with one implementation shape:

- one selected adaptive frame owns each source centre; no simultaneous
  full-band resolution layers are recombined
- the existing `512/1024/2048/4096` bank uses centered reflected reads,
  Hann/Hann windows, a native FFT per selected frame, and its exact diagonal
  dual
- native complex magnitudes remain unchanged; there is no magnitude smoothing,
  cross-resolution interpolation, blend, or gain match
- the fixed `4096` analytic spectrum is a decision surface only; ordered active
  peaks carry physical frequency and synthesis phase across frame-size changes
- physical frequency maps each active owner to its nearest native coefficient
  bin; surrounding native bins keep their current analysis-phase offset from
  that owner
- owner births initialize from current native analysis phase; frames without an
  active owner use current analysis phase instead of continuing dormant bins
- sample-refined transient anchors own exact source/output centres and reset
  active phase from the native frame
- the proven conflicted-bridge background substitution shares those anchors
  and removes bounded midpoint replicas without changing anchor samples
- one output timeline, exact target length, real DC/Nyquist, conjugate symmetry,
  and exact analysis-times-synthesis normalization remain invariant

This is a phase-coherent native coefficient path, not a new factor search. The
tracker never supplies synthesis coefficients. The shared-grid damage found by
Rule 30Y therefore does not re-enter through the auxiliary decision surface.
The first implementation stays report-only and must pass the prior mechanism
controls and complete `48`-row synthetic gate before any real-source render.

That implementation stops at the synthetic gate. It chooses an FFT per frame,
keeps real native magnitudes, projects fixed-grid physical-frequency owners to
native bins, retains native within-region phase offsets, and composes the exact
anchors and conflicted-bridge owner. Structure and event behavior pass. The
three stretched `55 Hz` rows do not: rendered frequency error reaches
`3.695086e-5` radians/sample while tracked-owner error stays at
`1.263528e-7`. All `300/300` active resolution transitions retain a matched
owner.

The active boundary was downstream of peak tracking and before final output:
native owner-bin/region phase projection, per-frame inverse synthesis, or its
exact-dual interaction. Source study supersedes that local repair. The failed
rows remain evidence, but the time-adaptive full-band representation is closed.

## Source-Studied Architecture Reset

Rubber Band R3 source resolves the multi-resolution ambiguity. Standard mode
runs long, middle, and short transforms simultaneously, but assigns each a
frequency interval. One full-band classification spectrum guides crossover,
reset, unlock, attack, and channel policy. The scale renders are not three
full-band copies and resolution is not selected once per time centre.

This corrects two earlier Signal decisions:

- the rejected union duplicated full-band synthesis across resolutions
- the replacement adapted one full-band resolution over time

R3 instead adapts exclusive frequency ownership while all scales remain on one
timeline. Its H/P/R classification is control evidence, not additive component
synthesis. Dynamic crossovers move to spectral valleys. Peak phase, reset,
unlocked high bands, low attack handling, and channel locking are separate
states inside one guide.

Signalsmith Stretch provides the required control architecture. It keeps one
long STFT and combines horizontal advance with weighted vertical predictions
from both directions and distances. This tests whether Signal needs multi-
resolution frequency ownership or primarily needs to replace hard owner-region
phase assignment.

The next Signal candidate is therefore one complete comparison:

- fixed-grid weighted multi-predictor control
- frequency-partitioned long/middle/short candidate
- one common time map and boundary contract
- classification-guided crossovers and explicit phase states
- existing current Signal, Signalsmith, and Rubber Band comparator evidence

No Rule 30AB repair, factor sweep, or per-metric follow-up chain is authorized.
The candidate is judged as a system on the complete synthetic and frozen mono
development gates before any promotion.

Concealed development listening rejects the frequency-partitioned path. Its
exclusive-scale topology did not transfer into a coherent Signal synthesis
system: stutter, double/soft transients, definition loss, and boundary clicks
repeat across rows. The fixed-grid weighted predictor is the only successor
direction retained. It repeatedly sounds clean, tight, or refined, confirming
that weighted multi-direction evidence is more promising than hard nearest-
owner phase replacement. Residual smear, grain, transient-shape variation, and
an end pop still block promotion.

No external parity conclusion follows from the first pack. External engines
were rendered from full stereo files, while Signal consumed isolated mono
excerpts. Exact-input confirmation must precede any Rubber Band or Signalsmith
ranking.

Batch 29.6CJ closes that integrity gap. One runner converts the frozen source
region into exact mono inputs, invokes both external engines, verifies their
complete output shapes, records file hashes, and then renders Signal from the
same decoded samples. The corrected pack is ready. No synthesis setting or
quality threshold changed.

Corrected short-form listening does not establish a weighted-predictor
successor. The path is best or tied on two rows and competitive on two, but
softness, smear, grain, and an end pop recur on four. Current Signal and Rubber
Band are more consistently safe. These sub-half-second sources primarily test
attacks and endpoints.

The last bounded decision is musical continuity: six five-second expansion
rows at `1.5x` or `2.0x`, comparing weighted predictor, current Signal, and
Rubber Band only. No setting changes. A non-win rejects the weighted
implementation rather than spawning local repair.

That decision is now complete. Weighted prediction improves on current Signal
in four of six long-form rows, removing much of its pervasive grain. It still
mutates one bass tone and produces severe phase damage on one sustained pad
row. Rubber Band remains best in four rows. The predictor family is therefore
validated, while the current implementation is rejected for promotion.

Source reinspection explains why this is not a dead end. Signal's proof used a
short `2048` transform at hop `128`, same-frame neighbour phase differences,
and one horizontal-plus-vertical magnitude-weighted sum. The studied topology
uses sample-rate-scaled 120/30 ms geometry, preliminary horizontal transport,
time-factor-scaled input-frequency twists from neighbouring output states,
energy normalization, and weak-prediction fallback. The next work corrects
that complete mechanism before any more corpus tuning.

The corrected Signal topology keeps a fixed 30 ms output interval and fourfold
centered transform support. Input centres follow the inverse fixed-ratio map;
an auxiliary input spectrum exactly one output interval behind the current
projected input centre drives horizontal phase transport. The preliminary
complex product is divided by the larger of previous and current input energy;
it is not target-normalized. Actual rounded input hops set the local time
factor. A separate
ascending-frequency pass combines short and transform/interval-distance
predictions from both directions. Each prediction observes fractionally sampled
input-frequency twists scaled by local time factor. Lower dependencies are
already corrected; upper dependencies remain preliminary. The result is
normalized to target input energy, with current-input fallback when combined
evidence is weak.

Signal uses its own square-root Hann/overlap normalization and arbitrary-length
RustFFT geometry. It does not copy the specimen's ACG window, fast-size planner,
control flow, or random diffusion. The complete synthetic gate directly covers
bass pitch, chord sidebands, transient replicas, silence/fallback, boundaries,
coverage, determinism, finiteness, and exact duration before real sources.

## Next Task

The complete report-only predictor fails the steady chord/pad sideband gate at
`-30.200611 dB` against `-60 dB`. Trace attribution places the first failure in
horizontal transport at `-28.182097 dB`, with a dominant spur one output frame
rate from the nearest tone. Exact overlap, normalization, and fallback are
excluded. Compare isolated and mixed horizontal observations before choosing
an observation-geometry or equation correction. Do not render real sources or
open local tuning.

That comparison assigns the next correction to the horizontal equation. All
four isolated tones create a frame-rate sideband despite nearly stationary
nearest-bin auxiliary ratios. Signal's translation target-normalized the
preliminary horizontal product; the studied topology divides by the larger of
previous and current input energy and defers target normalization until after
vertical re-prediction. Correct that one law before reconsidering observation
geometry.

The corrected preliminary energy law is retained but does not pass the gate.
Complete leakage changes only to `-30.236852 dB`; horizontal leakage improves
to `-29.975234 dB`; isolated tones remain dirty. The current horizontal trace
uses the previous frame's vertically corrected output state. Attribution must
separate direct horizontal recurrence from corrected-state feedback before
another topology correction.

State-lineage attribution excludes vertical feedback as a necessary cause. A
target-magnitude oracle carrying only horizontal phase state improves every
isolated tone but still produces one-frame-rate sidebands at `-41.444546` to
`-52.739473 dB`. Horizontal transport is an incomplete intermediate phase
field, so this result does not justify another local equation change. The next
architecture decision depends on the pinned complete upstream engine under the
same final-output measurement: either locate a Signal translation divergence
if upstream passes or revise the attainable target for this topology if it
does not.

Pinned complete-engine evidence resolves that fork. Signalsmith Stretch
`1.3.2` itself produces frame-rate leakage around `-45 dB` on isolated tones
and `-40.016259 dB` on the chord at `2x`; the prior `-60 dB` ceiling is not an
attainable fidelity requirement for this topology. Signal is nevertheless
`9.779 dB` worse on the paired chord and worse on three of four paired tones.
Use the absolute ceiling as a diagnostic. Gate translation fidelity against
the pinned complete engine on exact shared inputs before changing mechanism or
reopening musical comparison.

The frozen parity allowance is `1 dB` per exact quantized tone and chord. It
fails on three tones and the chord while all prior structural gates remain
closed and unchanged. Source inspection supplies the next bounded differential:
pinned fractional frequency lookup zero-extends out-of-range bins, while the
Signal translation clamps to the nearest edge. At `2x`, ten vertical
observations per frame differ near the low-frequency boundary. Ascending
correction can propagate those low-bin phase decisions upward, so boundary
policy is the next causal ablation. It is not yet accepted as the artifact
owner.

That ablation rejects boundary policy. Zero-extension changes isolated-tone
leakage by at most `0.033206 dB` and chord leakage by `0.068380 dB`; the same
three tones and chord still fail paired parity. The next comparison must align
internal source and Signal states at the analysis, preliminary-horizontal, and
corrected-output boundaries. Select the next candidate from the first measured
state divergence, not from another final-output guess.

The aligned trace finds that divergence before predictor transport. The pinned
engine's `960/240` support is represented by Signalsmith Linear revision
`56686735` as a `1024`-point modified real transform with `512` half-bin bands.
Signal represents the same support with a `960`-point standard real transform
and `481` bins. The bases differ in size, origin, and spacing, so raw bin phase
cannot be treated as an equation-level comparison. Test the modified half-bin
grid alone before revisiting prediction or windows.

The grid-only variant is identity-safe but worsens exact-input parity. It moves
the failure count from `[3 tones, 1 chord]` to `[4 tones, 1 chord]`; only the
`110 Hz` control improves. The modified half-bin representation is not a
standalone correction and remains report-only rejection evidence.

The remaining observed analysis differential is the window. Pinned
Signalsmith Stretch calls Linear's periodic Kaiser path at bandwidth
`block/interval = 4` and forces the `960/240` overlap product to exact
reconstruction. Signal uses square-root Hann with post-overlap normalization.
Test the pinned window alone on Signal's standard grid before considering any
interaction between representation choices.

The window-only result is also incoherent. It improves the `110 Hz` and
`220 Hz` controls, regresses the other two tones and the chord, and moves
paired failures to `[4, 1]`. The source coefficients prove that the even-length
Kaiser is periodic rather than endpoint-symmetric; the initial
confined-Gaussian configuration is overwritten by Stretch's explicit Kaiser
selection.

The bounded interaction closes source parity. Periodic Kaiser and the modified
half-bin transform each regress the translation when installed alone, but the
exact pair moves failures from baseline `[3 tones, 1 chord]` to `[0, 0]`.
Every combined tone is within `0.147 dB` of pinned source and the chord is
`0.641 dB` better. This confirms that weighted spectral prediction depends on
the complete analysis phase basis. Treat the window and grid as one coherent
representation, not independently selectable improvements.

The coherent representation also passes the complete synthetic system proof.
Bass and chord pitch, transient placement and replica protection, silence,
cancellation, boundaries, coverage, identity, mechanism exercise, and repeat
all pass while exact-input tone/chord parity stays closed. The representation
is now the report-only faithful-predictor research baseline. It is not a
production selection.

At `44.1 kHz`, the same source rules derive `5292/1323` support/interval and a
`6144`-point, `3072`-band modified half-bin transform. Six exact-input musical
rows pass length, finiteness, hard-integrity, and repeat gates. Relative to
pinned Signalsmith, coherent Signal improves timing and static residual on
four rows each, replica ratio on three, and boundary growth on none. The
representation therefore advances only to concealed listening, with boundary
behavior called out as a specific risk.

The concealed comparison is now frozen as six source references and twelve
level-matched trials. Its manifest, identity assignment, gains, audio files,
operator notes, and rate/channel/frame receipt repeat exactly. This is an
evidence artifact only. The coherent representation remains report-only until
all six rows have continuity, transient, grain/ringing, tonal, and boundary
findings.

The first pack applied the peak ceiling after choosing a raw-RMS target. That
made `M002` about `4.14 dB` unequal and shifted `M006` about `0.49 dB`. The
corrected exporter chooses a target reachable by every candidate and verifies
written-WAV pair RMS. Findings for `M001`, `M003`, `M004`, and `M005` remain
valid; `M002` and `M006` must be heard again.

The corrected record closes with five audible ties and one slight coherent-
Signal preference on `M003`. Coherent Signal has no listening loss against
pinned Signalsmith and remains the report-only source-studied baseline. This
resolves translation fidelity, not the product-quality target: the coherent
engine has not yet faced Rubber Band on these exact long sources.

Batch 29.6DB reuses the same six exact mono inputs and corrected level matcher
for coherent Signal versus Rubber Band R3 `4.0.0`. Both paths repeat and pass
hard integrity. Concealed listening finds Signal cleaner on `M002` and `M004`,
slightly cleaner on `M005`, and tighter but marginally grainier on `M001`.
Rubber Band is cleaner on `M003` and `M006`. Defects change sides with material;
neither engine wins overall. The coherent Signal path is competitive on this
frozen mono set and advances unchanged to shared-decision linked-stereo proof.
Dynamic ratio, routing, and promotion remain closed.

## Relationship-Preserving Linked Stereo

The first linked renderer shared timing, geometry, traversal, and aggregate
corrected/fallback choice while retaining one phase recurrence per channel. It
passed structural mechanics but failed quadrature phase, expansion delay, and
unequal-correlated image controls. Independent mono recurrence reproduced every
failure mask. Shared scheduling alone is insufficient: arbitrary interchannel
phase must be explicit in synthesis.

Primary-source research converges on reference-relative recurrence. Signalsmith
Stretch selects the greatest-energy channel per bin, completes phase prediction
there, and derives peer output through the peer/reference current input complex
relation. The 2005 AES multichannel TSM paper specifies the same ownership
shape: update the greater same-bin peak first, then preserve the original phase
relationship in the lesser peak. Rubber Band R3 independently corroborates a
greatest-channel tracked trajectory plus current analysis-relative offset, but
its GPL expression, guidance, and constants remain outside Signal.

Signal's fixed-ratio two-channel architecture therefore uses one per-bin
reference recurrence. The greater current target energy selects the reference,
with lower channel index resolving an exact tie. The peer retains
its own magnitude and takes the reference output phase plus its wrapped current
input phase difference from that reference. Exactly silent peers remain zero.
If reference prediction fails the existing viability test, its current-input
fallback makes peer projection land on peer current-input phase. No new
threshold, hysteresis, mid/side transform, sample crossfeed, or peer-magnitude
borrowing is introduced.

This topology preserves a relationship; it does not copy a dominant phase.
Schedule, geometry, traversal, crop, and overlap remain shared. Spectra,
magnitudes, accumulation, and normalization remain channel-owned. Mono code and
hashes remain frozen.

The next report-only proof must exercise both reference owners, an exact tie,
and an ownership crossing before repeating the unchanged phase, delay, image,
transient, replica, and crossfeed gates. Switch instability is a stop condition,
not permission to tune hysteresis. Stereo listening, dynamic ratio, realtime,
cache, and product routing remain closed.

That proof passes mechanics and materially improves stereo fidelity without
passing the gate. Both channels own bins, owner crossings introduce no local
step growth, delay is exact at all three ratios, and correlated-image damage
drops from roughly `12 dB` to at most `0.434087 dB`. Residual quadrature IPD is
`0.007623` to `0.016074 rad`, above the unchanged `1e-9 rad` ceiling. The first
two ratios also remain slightly above the `0.25 dB` mid/side ceiling.

The next architecture decision depends on stage attribution. Measure relation
error after coefficient projection and real-edge constraint, compare whole and
interior synthesis, and use a known-constant-relation oracle to distinguish
current per-bin relation variability from overlap or boundary effects. Do not
infer a peak-region policy from the residual before that trace.

That trace excludes the coefficient and real-edge stages at
`4.440892e-16 rad`. Boundary removal reduces quadrature IPD by roughly one to
two orders of magnitude, but steady interior image damage remains. A fixed
quadrature oracle does not consistently improve the output, so current per-bin
relation variability is not a sufficient explanation. The remaining seam is
post-spectrum: inverse synthesis, overlap accumulation, normalization, or the
finite-record measurement. No topology change is authorized until those four
are separated.

Synthesis closure calibrates the finite-record measurement and locates the
first remaining seam. Ideal whole records are effectively exact; cropped ideal
records carry `0.000142` to `0.000489 rad` measurement bias. After calibration,
real support-frame synthesis already carries relationship error, overlap often
reduces it, and normalization is neutral within `1e-9 rad`. The current
per-channel real-frame inverse therefore loses information needed to make a
frame-local complex relation close across windowed overlap.

The next bounded topology test accumulates positive-frequency analytic frames
as complex values through the same window and overlap before taking real
output. It is an ablation, not an adopted renderer. Current output and all
recurrence, magnitude, scheduling, geometry, crop, and normalization policy
remain frozen while mechanics and stereo quality decide whether the synthesis
representation is viable.

That ablation is rejected and corrects the causal interpretation. Complex and
real overlap have exactly equal phase metrics and effectively equal image
metrics. Their samples differ only at `2.220446e-16` to `3.330669e-16` from FFT
rounding. Real support synthesis exposes the residual but does not cause it.

The remaining evidence gap is coefficient coverage. The exact relation trace
measured only post-initial frames where both channels exceeded a relative
energy threshold; it did not close first-frame, recurrence-fallback, or weak-bin
contributions. Those classes can still accumulate into a measurable whole
output. Full contribution attribution must isolate them before another phase or
synthesis topology is credible.

That attribution finds no omitted coefficient owner. Initial, corrected,
fallback, significant, and weak classes all preserve their input relation
within `4.440892e-16 rad`. Fallback is nearly absent and energetically
negligible. Weak coefficients are numerous but carry below `0.00053%` of total
energy. Constant-relation forcing of weak coefficients worsens phase; fallback
is neutral; initial-frame forcing is inconsistent. None moves correlated image
materially.

The residual is therefore no longer assigned to a coefficient, edge,
synthesis, overlap, normalization, or boundary implementation defect under the
current proof. Calibration separates finite-record measurement floors from
material Signal drift. Render-wide normalized-Gram coloring then closes only
the aggregate statistic: it fails calibrated tone and interior-image gates and
local consistency. The next architecture step studies Rubber Band's verified
linked-channel source and behavior before selecting another synthesis-time
invariant.

## Next Task

Run Batch 29.7L Rubber Band linked-stereo mechanism study. Promote at most one
license-safe synthesis-time invariant before another repair.
