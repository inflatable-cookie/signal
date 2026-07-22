# Creative Stretch Source Triangulation

Status: reviewed; direct-renewal reset authorized and complete brief frozen
Owner: dsp
Updated: 2026-07-22
Roadmap: `g10.031`, Batches 31.16, 31.30-31.41, 31.64-31.65

## Question

Which source-available render paths actually produce the retained creative
targets, and what do they change about Signal's next architecture decision?

This study covers complete pinned paths, not isolated papers or parameter
ideas. It does not copy source expression, constants, thresholds, tables,
masks, or control flow.

## Pinned Specimens

| Specimen | Revision | Licence posture | Retained target |
| --- | --- | --- | --- |
| PaulXStretch | `v1.6.0`, `8ec191fdd7203354c79391cbc04c9fd83fa30ea0` | GPL repository; clean-room evidence only | neutral `Dream` |
| CDP8 | `CDP8.0`, `456ffe0687c8d8206f8bc4e22273587db4c0ee0a` | LGPL-2.1-or-later; clean-room evidence only | `Spectral` |
| Potenza Time Stretch | `ddb44a8f949b3f49320932e1d2e997b3a02149bb` | GPL-3.0; clean-room evidence only | Akai-style `Cyclic` |
| SiTraNoStar | `v2.0.1`, `2edf7b693040b5070116299973abf83dc5ba86e5` | GPL-3.0; clean-room evidence only | component-owned neutral `Dream` study |

The existing comparator pack already contains PaulXStretch and CDP at `4x`,
`8x`, and `16x`, plus REAPER's proprietary ReaReaRea cyclic reference at
`4x`, `8x`, and `16x`. This study connects those audible outputs to source
mechanisms.

## Whole-Path Comparison

| Boundary | PaulXStretch | CDP `SPECTSTR` | Potenza |
| --- | --- | --- | --- |
| source clock | fractional accumulator controls when the source window advances | output analysis-frame positions index the input analysis timeline | one slow ideal cursor plus unit-rate grain readers |
| representation | long-window magnitude spectrum | amplitude and instantaneous-frequency analysis frames | native waveform samples |
| time expansion | repeat or slowly move the analysed magnitude view | interpolate between adjacent analysis frames on a denser output grid | reduce new-grain anchor advance while each grain reads at unit rate |
| synthesis character | new stochastic phase per spectral frame, then frame crossfade | phase-vocoder resynthesis of interpolated amplitude/frequency frames | two overlapping waveform grains and direct crossfade |
| default transient owner | none in the captured neutral default | none in `SPECTSTR` | none |
| characteristic fault or feature | smooth phase-forgotten smear | exposed vocoder separation and frequency-track decoherence | metallic or cyclic repetition |
| stereo evidence | per-channel engines; source schedule and onset trigger shared, phase draws separate | retained comparator is mono and duplicated for pack shape | caller-owned; the struct has no linked-channel policy |
| output boundary | synthesis extension beyond nominal duration | analysis/synthesis extension beyond nominal duration | caller-owned |

These are three different owners. One parameterized kernel should not pretend
they are coefficient settings of the same recurrence.

## PaulXStretch

### Render path

The neutral renderer is materially simpler than Signal's rejected diffusive
briefs:

1. Keep a rolling source history and select one long analysis view from the
   fractional stretch accumulator.
2. Window and transform that view.
3. Retain magnitudes and discard input phase.
4. Rebuild every active spectral bin with a newly generated phase.
5. Inverse transform.
6. Crossfade the new frame with the preceding output frame.
7. Advance the source accumulator by inverse stretch; request new source input
   only when the accumulator crosses a source block boundary.

The captured default disables onset handling and every optional spectral
processor. Spread, tonal/noise selection, harmonics, pitch/frequency shift,
ratio mixing, filtering, and compression therefore do not explain neutral
`Dream`. Long support, phase forgetting, slow magnitude motion, and output
frame blending do.

The UI's `16384` FFT setting names the core buffer control. The pinned core
constructs a transform over twice that sample support. Signal must treat the
capture label as a product setting, not blindly transplant it as transform
geometry.

### Scheduling and boundaries

The source accumulator owns duration. The synthesis frame cadence stays
regular while source windows repeat or move slowly at large ratios. Optional
onset handling can force a new source buffer and later repay the timing credit,
but it is inactive in the retained comparator.

PaulXStretch estimates nominal output duration with an added synthesis tail.
The retained Signal comparator crop removes that tail. Exact target length is
therefore a Signal-owned boundary, not an upstream property.

### Stereo consequence

Each output channel owns a separate stretcher and random generator. Channels
share input scheduling and the maximum onset decision, but they do not share
one spectral phase field. This can produce a pleasing diffuse image, but it
does not meet Signal's linked-channel contract by itself.

Signal must own a new linked excitation rule. It must preserve mono and stable
image relationships without recreating the rejected exact complex-relation
proof. Independent upstream phase draws are evidence of target character, not
permission for unrelated Signal channel trajectories.

### Source evidence

- [magnitude analysis and stochastic phase synthesis](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L109-L263)
- [rolling analysis, frame synthesis, crossfade, and source accumulator](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L320-L563)
- [per-channel stretcher ownership and shared onset scheduling](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/StretchSource.cpp#L331-L399)
- [default controls and disabled optional processors](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PluginProcessor.cpp#L150-L230)

## CDP `SPECTSTR`

### Render path

CDP uses an explicit three-stage pipeline:

1. `pvoc anal` converts the waveform to amplitude and instantaneous-frequency
   analysis frames.
2. `SPECTSTR` maps each output frame to a fractional input-frame position and
   linearly interpolates both fields between adjacent frames.
3. Optional decoherence sorts bins by amplitude and perturbs the frequency
   field of a selected low-amplitude subset.
4. `pvoc synth` reconstructs the waveform through phase-vocoder overlap-add.

The retained `d-ratio=1`, `d-rand=0.5` values describe the external comparator
only. They are not Signal constants.

### Character consequence

CDP's vocoder colour does not come from PaulX-style phase replacement. It comes
from dense interpolation of analysis amplitude/frequency frames followed by
phase-vocoder resynthesis. Decoherence is a controlled modifier applied after
interpolation, biased toward lower-energy bins.

This supports `Spectral` as a separate owner. It does not support using CDP
interpolation inside neutral `Dream`, where the operator rejected exposed
vocoder colour.

The retained CDP output is mono. Duplication to stereo in the listening pack
was file-shape normalization, not linked-stereo evidence.

### Source evidence

- [frame map and amplitude/frequency interpolation](https://github.com/ComposersDesktop/CDP8/blob/456ffe0687c8d8206f8bc4e22273587db4c0ee0a/dev/science/spectstr.c#L1173-L1347)
- [low-amplitude frequency decoherence](https://github.com/ComposersDesktop/CDP8/blob/456ffe0687c8d8206f8bc4e22273587db4c0ee0a/dev/science/spectstr.c#L1642-L1685)
- [PVOC analysis and synthesis](https://github.com/ComposersDesktop/CDP8/blob/456ffe0687c8d8206f8bc4e22273587db4c0ee0a/dev/pv/pvoc.c)

## Potenza

Potenza implements one compact Akai-style topology:

- two forward or reverse unit-rate waveform readers
- a slow total source cursor
- new-grain anchors separated according to the stretch ratio
- direct overlap crossfade between the two readers
- optional pitch compensation owned by read speed

There is no waveform-similarity search, pitch detector, transient detector,
spectral correction, linked-channel rule, exact-length owner, or quality gate.
The audible cyclic or metallic result is a consequence of overlapping
different source phases, not an error repaired elsewhere in the algorithm.

Signal's rejected `CyclicGrain` was a valid clean-room translation of this
family. Its `20.778`-cent synthetic result proves it missed the frozen Signal
tone gate; it does not prove that the topology failed to resemble the Akai
target. The missing `2x` ReaReaRea comparison means that question was never
answered. The rejected candidate stays rejected. Any future cyclic reopening
must calibrate character metrics against the reference before implementation.

Source evidence:
[two-grain scheduling and crossfade](https://github.com/dar-io-p/potenza-time-stretch/blob/ddb44a8f949b3f49320932e1d2e997b3a02149bb/TimeStretch.h).

## Reinterpretation Of Signal's Rejections

| Signal family | Source comparison | Result |
| --- | --- | --- |
| `DiffuseSpectral` | added instantaneous-frequency carrier, correlated diffusion, log-magnitude evolution, and fourfold normalized overlap absent from neutral PaulXStretch | not a faithful topology test of the preferred reference |
| `ContinuousExcitationSpectral` | moved farther toward continuous full-complex recurrence while PaulXStretch deliberately forgets phase every frame | closed as tested; not evidence against frame-renewal synthesis |
| direct-complex relation | demanded an exact algebraic relation from a target whose channels use separate phase draws | proof contradiction must not become the next stereo model |
| `CyclicGrain` | represents the Potenza family | valid family rejection against its frozen gate; target resemblance remains unheard |
| `SimilarityAlignedCyclic` | changed to WSOLA-like continuity search absent from Potenza | a different family, not a closer Akai translation |

The process tested complete candidates correctly against their frozen briefs.
The briefs did not first match the simplest complete source path behind the
preferred `Dream` target. This study supplies new architecture evidence; it
does not authorize tuning or restoring any rejected branch.

## Batch 31.16 Signal Decision

Reopen `Dream` research around one source-backed family:
`RenewalSpectral`.

Its complete brief must freeze one Signal-owned renderer with:

- one exact monotonic source/output map and fixed target length
- one long-window output-synchronous magnitude analysis
- no neutral instantaneous-frequency carrier, phase propagation, magnitude
  slew, transient detector, or continuous-excitation recurrence
- deterministic stochastic phase renewal at each synthesis frame
- one bounded frame-combination and normalization law that owns crest behavior,
  exterior support, and exact crop
- one linked-channel excitation law shared by analysis decisions and random
  state while preserving native channel content
- bounded duration-independent memory and deterministic offline cost
- structural hard limits followed by comparator-calibrated synthetic controls,
  concealed long-form mono listening, then independent stereo listening

This is materially different from every rejected spectral brief. The new
evidence is the end-to-end PaulXStretch source path matched to the retained
preferred comparator.

`RenewalSpectral` owns neutral `Dream` only. Do not force other anchors into its
first candidate:

- `Spectral` later uses a separate CDP-like amplitude/frequency-frame owner
- `Cyclic` stays closed pending a separately authorized comparator-calibrated
  reopening
- `Rough` remains a behavioral target without public source backing
- `Cloud`, routing, blends, dynamic ratio, cache, public API, Loophole, and
  Chorus remain closed

The stable `duration`, `character`, `motion`, `detail`, `space`, and `seed` UI
vocabulary remains valid. Internal owners may change or blend later without
exposing algorithm names.

## Crest Ownership Reassessment

Batch 31.19 reconciles the two neutral-`Dream` crest failures:

- `DiffuseSpectral` measured `7.08 dB` crest growth after independently
  diffusing bin phase.
- `RenewalSpectral` removed the carrier, magnitude recurrence, and normalized
  overlap-add, yet measured `8.263162 dB` after complete independent phase
  renewal.

The second result isolates the shared cause. Independent stochastic bin phase
does not own the crest of their waveform sum. PaulXStretch demonstrates a
musically useful member of that family, but its pinned implementation and seed
do not establish a portable crest bound for a clean-room Signal renderer.
Changing Signal's phase distribution, window, hop, gain, crossfade, or seed
would repair the rejected family rather than select a new owner.

The remaining public techniques do not supply a build-ready whole renderer:

| Technique | Crest mechanism | Missing neutral-`Dream` ownership |
| --- | --- | --- |
| low-crest multisine phase design | jointly selects phase to reduce periodic peak factor | source-mapped, nonstationary musical stretch; linked stereo; bounded fixed cost |
| iterative arbitrary-spectrum phase optimization | numerically minimizes crest for a prescribed spectrum | deterministic iteration bound, evolving source map, retained musical evidence |
| IAAFT surrogate synthesis | iterates between a target spectrum and amplitude distribution | local time map, exact event order, linked stereo, long-form stretch evidence |
| STN noise morphing | resynthesizes a separated residual from a moving magnitude envelope | complete first-party separator and whole-mix renderer; tonal and stereo ownership |
| bounded continuous excitation | owns one full-complex waveform rather than independent bins | already rejected in Signal on linked-channel relation ownership |

Low-crest phase design and IAAFT are valid separate research programs, not
candidate-ready replacements. Noise Morphing is a component inside an STN
system, and Signal already tested the bounded continuous-excitation translation
without finding a linked-stereo owner. None supplies one source-backed path
through crest, linked stereo, exact length, bounded state, and the retained
PaulX-centred musical target.

Batch 31.19 therefore closed neutral `Dream` without promotion. The operator
rejected that closure. It had treated `3.88 dB` from long-form PaulXStretch
musical rows as calibration for a synthetic uniform-noise stop row that was
never rendered through PaulXStretch. It also treated Signal's substituted
equal-power frame blend as a conclusive test of a source path that uses a
raised-cosine blend plus position-dependent amplitude-modulation compensation.

The candidate rejection stands; the family closure does not.

Batch 31.20 completed the missing comparison. Pinned PaulXStretch
worst-channel uniform-noise crest growth measured `9.932`, `11.899`, and
`10.432 dB` at `4x`, `8x`, and `16x`. The rejected Signal row was
`8.263162 dB` at `4x`. The old `6 dB` ceiling was not a PaulX-relative
character gate.

The whole-path compensation distinction remains useful, but not as a promise
of low crest. Signal derives `c=1/sqrt(a^2+b^2)` from the variance of two
equal-energy uncorrelated frames under complementary raised-cosine weights.
This removes deterministic blend-position energy modulation. It does not
bound stochastic waveform peaks.

The complete clean-room decision is frozen in
[Offline Creative CompensatedRenewalSpectral Renderer Brief](../../architecture/offline-creative-compensated-renewal-spectral-brief.md).
No upstream constant, threshold, random generator, or control flow transfers.

Primary evidence:

- [Schroeder, low-peak-factor phase selection](https://doi.org/10.1109/TIT.1970.1054411)
- [Yang et al., arbitrary-spectrum crest minimization](https://pubmed.ncbi.nlm.nih.gov/25832418/)
- [Schreiber and Schmitz, IAAFT surrogate data](https://doi.org/10.1103/PhysRevLett.77.635)
- [Noise Morphing for Audio Time Stretching](https://arxiv.org/abs/2312.14586)

## Admission Correction

Future creative briefs must separate three classes of evidence:

1. Hard integrity: exact length, finiteness, determinism, bounded state,
   exterior continuity, no clipping, no dropouts.
2. Character diagnostics: reference-calibrated pitch, crest, periodicity,
   smear, roughness, and image measures. These reject a direction mismatch but
   do not replace listening.
3. Promotion: concealed retained-pack listening, with independent stereo review
   after mono admission.

Do not inherit transparent thresholds or exact samplewise stereo algebra unless
the selected creative representation and product invariant both require them.
Every numeric gate must cite the retained comparator row or a hard safety
boundary before implementation.

## Seed Authority Reassessment

Batch 31.29 passed construction and structural admission, then failed one
`16x` replica row and two `4x` pitch rows. That does not select a range-aware
replacement:

- pinned PaulXStretch keeps one transform geometry, fractional source
  accumulator, magnitude-renewal path, and adjacent-frame blend across the
  retained ratios
- Batch 31.25 and Batch 31.29 specify the same Signal mono transform, map,
  phase address, blend, sources, and metrics
- both briefs expose seed as a request field but freeze no candidate seed for
  synthetic admission
- Batch 31.29's helpers chose seed `17`; Batch 31.25's passing receipt records
  no seed

Stochastic output cannot support a comparative architecture decision when its
request identity differs or is unknown. The Batch 31.29 checkpoint remains
rejected under its frozen tests. Its failure does not prove that fixed
resolution or one renewal owner fails across `4x` through `16x`.

The clean-room correction is evidence ownership, not a random-seed sweep.
`SeedAuditedSourceRelativeRenewalSpectral` retains one source-backed renewal
path and freezes the existing audited address seed as `ADMISSION_SEED` for
every synthetic and listening candidate render. Public seed/reroll exposure
still requires later multi-seed character review.

Batch 31.31 tested the audited address seed once. Construction and all `15`
structural owners passed. `Y04` passed both impulse sources at all ratios, but
`Y02` failed the `8x` chord at `13.351828347` cents against an
`11.331375778`-cent ceiling. This resolves the seed-authority contradiction:
the replica miss was seed-sensitive, while tonal pitch remains unreliable.

Batch 31.29 and Batch 31.31 now fail the same tonal-pitch class across two
seeds and different ratios/material. Another seed or scalar variant is not new
research. The next study must find a materially different whole-renderer owner
for tonal coherence or close the renewal family.

Batch 31.32 found no eligible complete owner. Signalsmith's coherent predictor
uses an author-identified randomized fallback above `2x`; the retained peak,
material-state, sinusoidal, STN, and classical-toolbox paths reopen rejected
families or separate output owners. TSM-Net adds public pretrained inference,
but not a released training path, usable repository licence, or intrinsic
linked pitch law. Renewal is closed. PaulXStretch remains the target reference;
the target itself is not closed.

The operator then changed the product gate rather than the renderer. Finite
PaulX-relative pitch delta is now mandatory diagnostic evidence; concealed
listening decides whether it harms neutral `Dream`. This preserves the pinned
source conclusion that PaulX uses one renewal path across all retained ratios.
It authorizes one fresh source-relative candidate without changing the source
translation, seed, or any terminal integrity and stereo boundary.

Batch 31.34 passed construction `1/1` and structural `15/15`, then failed
synthetic `Y08` on exact-zero impulse hops at every ratio. The frozen
executable assertion used complete impulse output while the normative dropout
boundary names mapped non-zero support. The candidate was rejected and
deleted. Batch 31.35 classified that mismatch as executable evidence-
construction failure and froze a fresh support-audited identity. Source
triangulation, renderer formulas, and terminal thresholds remain unchanged.

## Stereo Ownership Reassessment

Batch 31.36 passed every objective and concealed mono gate, then failed the
source-relative stereo gate at `16x`. Whole-render and band balance stayed
close, but mapped local windows reached `9.418990 dB` balance error and
reversed channel dominance on full mix. This follows Batch 31.25's global
balance inversion under the earlier mid/side law.

The native-channel pair law is already the strongest current-frame common-
rotation translation available inside renewal. At `space=0`, it retains both
channel magnitudes and their exact analyzed complex relation while replacing
only common phase. This is algebraically the current-frame part of Bungee's
common region rotation. The failure occurs after inverse synthesis and
adjacent-frame blending because every frame receives a new unrelated common
phase. Coefficient relation does not own interference between successive
waveforms, so local channel energy can drift even when every frame is exact.

Retained complete-source evidence supplies no eligible correction:

| Source family | Potential stereo owner | Why it is not a renewal successor |
| --- | --- | --- |
| PaulXStretch | shared schedule with separate channel phase draws | defines the preferred diffuse target but no source-relative linked image |
| Bungee | predecessor-driven common peak-region rotation | adds temporal phase and peak state; the current-frame invariant alone already failed |
| Signalsmith | horizontal/vertical prediction with one reference channel | coherent temporal recurrence, incomplete above `2x`, not phase-forgetting renewal |
| Rubber Band R3 | conditional compatible peak-trajectory sharing | returns to tracked peaks, material states, and multiresolution phase-vocoder ownership |
| SBSMS | paired partial clocks and direct oscillators | pinned source already failed mono quality, local stereo, and mechanics feasibility |
| covariance or consistency projection | iterative spatial/output constraint | post-hoc repair without a supported feasible set, bound, or transparent source target |

Windowed channel gain, relation smoothing, dormant relation state, another
phase address, or a different `space` curve would repair the rejected
candidate without changing its missing temporal waveform owner. None is
eligible under Contract `084` Rule 7.

Close the renewal family without promotion. Keep the PaulX-like `Dream` target
and comparator evidence. A future renderer requires new complete-system
evidence for one temporal linked-waveform owner across mono character, stereo,
exact length, determinism, and bounded state.

PaulXStretch's independent channel engines expose a separate product question:
whether creative stereo must remain source-relative in every local window, or
whether bounded integrity plus comparator-relative independent listening owns
promotion. That is a Contract `085` operator decision, not an architecture
finding and not permission to revive Batch 31.36.

The operator selected the second policy after Batch 31.37. Source evidence now
has a consistent role: PaulX defines the diffuse musical target, Signal keeps
one shared schedule and random trajectory plus hard whole/band balance, and an
eligible independent listener decides whether the resulting creative image is
competitive. Local mapped-window source balance remains measured for both
Signal and PaulX but is diagnostic.

This policy reopens one fresh documented candidate, not the deleted checkpoint
and not a stereo repair experiment. The renderer remains the tested linked
native-channel renewal law. Its complete fresh authority is
[ComparatorAuditedRenewalSpectral](../../architecture/offline-creative-comparator-audited-renewal-spectral-brief.md).

Batch 31.39 implemented that authority once from fresh source. Construction
and structural admission passed, but synthetic `Y04` failed one `16x` replica
row and `Y09` failed linked-stereo swap at `4x` and `8x`. The candidate was
deleted before listening. Batch 31.36 passed both owners under the nominally
same formulas and seed. That receipt divergence must be reconciled before the
source evidence can support another implementation decision; Batch 31.40
records that reconciliation below.

Batch 31.40 found that the shared authority stops at formulas, counter values,
seed, support tables, and gate inventory. Candidate source, helper bodies,
assertions, per-row values, and output digests were deleted. `Y04` has a
correctable prose error in the Batch 31.39 closeout: `-30 dB` is its active
window threshold, not a secondary-peak ceiling. `Y09` has no frozen executable
source-relative swap assertion after exact time-domain swap was disclaimed.

The receipts therefore cannot be compared as identical executable evidence.
Recreating that identity would require a new brief and a third renewal
candidate. Source triangulation supplies no materially different complete
renewal owner to justify that work. Renewal closes; PaulXStretch remains the
retained behavioral target rather than an admitted Signal implementation.

## Batch 31.41 STN Source Reassessment

The operator commissioned research for a different complete creative owner.
One newly reviewed public implementation changes the earlier STN conclusion:
[SiTraNoStar `v2.0.1`](https://github.com/ollpu/SiTraNoStar/tree/2edf7b693040b5070116299973abf83dc5ba86e5)
is a runnable GPL-3.0 application, not only a component paper. It supplies an
end-to-end mono path through two-stage sines/transients/noise decomposition,
identity-phase-locked tonal synthesis, transient relocation, noise morphing,
component recombination, and file export. It is clean-room evidence only.
Signal must not copy its expression, constants, thresholds, or control flow.

The implementation also makes its limits concrete:

- it reads and decomposes only the first input channel, then duplicates the
  mono mix to the second playback channel
- its random generator is initialized from `random_device`, so repeated
  renders are not deterministic
- its source-frame count and synthesis extension own approximate duration,
  then playback wraps; there is no Signal-style exact-length boundary
- decomposition materializes full-file transforms rather than bounded
  duration-independent working state
- the public control range stops at `10x`; `16x` is not demonstrated

Those are missing product contracts, not reasons to discard the material
model. The source path is materially different from renewal: stochastic
excitation owns only the separated noise residual. Tonal energy keeps a
phase-propagating peak owner, and transient waveform segments move once on the
same output map. Renewal instead discarded phase across the complete mixture
and left adjacent-frame waveform interference responsible for pitch, replica,
crest, and stereo behavior.

The published evidence completes the architecture triangulation:

- [Enhanced Fuzzy Decomposition](https://arxiv.org/abs/2210.14041) supplies a
  reconstructing two-stage soft-mask separation with simultaneous long tonal
  and short transient resolution.
- [Noise Morphing](https://arxiv.org/abs/2312.14586) supplies continuous white
  excitation shaped by the time-interpolated residual log spectrum. Its blind
  test covers mono material at `2x`, `4x`, and `8x`; it explicitly leaves
  stereo and multichannel extension as future work.
- [Extreme Audio Time Stretching Using Neural Synthesis](https://arxiv.org/abs/2211.16992)
  validates the same whole STN scheduling pattern at `4x` and `8x`, including
  transient relocation, a source-envelope correction before recombination,
  and independently processed stereo channels. Its unreleased training path,
  weights, and expensive autoregressive inference exclude the neural residual
  synthesizer from Signal.
- SiTraNoStar supplies executable classical noise-morphing evidence without a
  neural dependency. Its mono-only I/O means it does not supply Signal's
  linked-stereo law.

No single upstream artifact satisfies Contract `085`. Together they support
one clean-room complete architecture family whose missing ownership can be
frozen by Signal without inventing another renewal repair.

## Selected Family: `LinkedStnNoiseMorph`

Batch 31.41 selects `LinkedStnNoiseMorph` for one complete docs-only brief.
The brief must define one renderer, not interchangeable component options:

1. One exact monotonic output-to-source map drives every component.
2. One channel-symmetric two-stage soft-mask analysis separates tonal,
   transient, and residual material while retaining each native channel.
3. Tonal peaks own persistent phase trajectories and linked-channel phase
   relations. Stochastic renewal may not touch the tonal lane.
4. One shared transient state machine detects, segments, places, and emits
   each native-channel event once. It owns collisions, seams, and replica
   prevention.
5. One continuous deterministic multichannel excitation is shaped by the
   interpolated residual spectrum. A time-varying stereo cross-spectrum or an
   equivalent explicit relation law owns residual width and balance; separate
   unrelated channel noise is forbidden.
6. One mapped source-envelope law may shape the tonal-plus-noise bed before
   native transient recombination. It must own the entry/tail distribution
   observed in the operator comparison without adding an arbitrary exterior
   fade.
7. Windowing, component reconstruction, normalization, exact crop, boundaries,
   bounded memory, deterministic seeding, and cleanup are one synthesis
   system.

This family plausibly addresses the retained audible gap as a system:

- persistent tonal phase removes renewal's pitch instability and atonal
  cross-bin ringing from the tonal component
- one-shot transient relocation attacks softened attacks, visible replicas,
  and event-placement drift directly
- residual-only noise morphing keeps desired dream-like diffusion without
  turning bass and pitched energy into extra low-end noise
- shared masks, events, tonal trajectories, and residual spatial statistics
  prevent independent channel classification and excitation from owning the
  stereo image
- mapped envelope correction and exact crop give start and tail energy to the
  renderer rather than to accidental synthesis support

The selection is not a quality claim. Source listening used short mono clips,
mostly environmental material, and stopped at `8x`. The Signal target includes
long-form music, `16x`, linked stereo, exact output length, and deterministic
bounded execution. Those are terminal admission risks.

Other paths remain ineligible:

- neural STN has no released complete training/weight authority and adds a
  production model dependency
- Loris, SMS Tools, and SBSMS expose useful additive or residual mechanisms but
  no newly qualified complete linked creative path; SBSMS remains source-
  feasibility rejected
- Signalsmith, Bungee, Rubber Band, and the frozen Signal renderer are coherent
  stretch references, not a PaulX-like residual-morph owner
- CDP remains the separate vocoder-like `Spectral` target; cyclic, routing,
  product exposure, Loophole, and Chorus stay closed or paused

Primary source audit:

- [SiTraNoStar history and complete application](https://github.com/ollpu/SiTraNoStar/blob/2edf7b693040b5070116299973abf83dc5ba86e5/README.md)
- [two-stage STN decomposition](https://github.com/ollpu/SiTraNoStar/blob/2edf7b693040b5070116299973abf83dc5ba86e5/Source/STNDecomposition.cpp)
- [component synthesis and noise morphing](https://github.com/ollpu/SiTraNoStar/blob/2edf7b693040b5070116299973abf83dc5ba86e5/Source/TSM.cpp)
- [mono input, component recombination, and export](https://github.com/ollpu/SiTraNoStar/blob/2edf7b693040b5070116299973abf83dc5ba86e5/Source/MainComponent.cpp)
- [SiTraNoStar GPL-3.0 licence](https://github.com/ollpu/SiTraNoStar/blob/2edf7b693040b5070116299973abf83dc5ba86e5/LICENSE)

## Batch 31.64 Simpler-Owner Reassessment

Later linked-STN work closed without acoustic evidence. Rechecking the pinned
whole paths found no unused, materially simpler fifth family:

- CDP remains vocoder-like `Spectral`
- Potenza and WSOLA-like paths remain cyclic waveform overlap
- coherent PV and sinusoidal paths retain phase and own another character
- image inversion and learned synthesis add iterative, full-file, model,
  licence, stereo, or cost boundaries
- STN is the already-closed complex component family

The simplest path that owns the preferred sound is still the pinned PaulX
magnitude-renewal core. Signal already matched it in concealed mono listening
and reached solid speaker stereo with bounded balance, low-frequency-noise,
and entry/tail differences. Reopening therefore requires an evidence-backed
product-gate change, not a renamed new algorithm or recovered candidate.

The architecture decision is
[Offline Creative Direct-Renewal Owner Study](../../architecture/offline-creative-direct-renewal-owner-study.md).

Batch 31.65 records the operator-authorized reset and freezes one complete
candidate authority:
[Offline Creative DirectRenewalDream Renderer Brief](../../architecture/offline-creative-direct-renewal-dream-brief.md).

## Next Task

Run Batch 31.66 only. Implement the complete `DirectRenewalDream` authority in
its exact isolated worktree. Do not recover a renewal checkpoint or alter the
frozen brief.
