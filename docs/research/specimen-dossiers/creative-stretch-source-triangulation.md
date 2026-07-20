# Creative Stretch Source Triangulation

Status: reviewed; seed-audited PaulX-like `Dream` authority active
Owner: dsp
Updated: 2026-07-20
Roadmap: `g10.031`, Batches 31.16 and 31.30

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

## Next Task

Run `g10.031` Batch 31.31 only. Implement the frozen seed-audited brief once in
its named disposable worktree. Complete construction before structural
admission and keep `ADMISSION_SEED` immutable.
