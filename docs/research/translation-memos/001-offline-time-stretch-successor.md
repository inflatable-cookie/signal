# Offline Time-Stretch Successor

Status: promoted
Memo: `g10.029` structural reassessment
Owner: dsp
Last updated: 2026-07-10
Related roadmap: `g10.029`

## 1) Project Problem Statement

Signal's first structural hybrid rendered independent `1024`, `2048`, and
`4096` STFT outputs, then attempted time-domain ownership crossfades. Only
`56/2024` ownership spans passed the frozen transition gate. Bounded lag
analysis recovered some local correlations only with large and inconsistent
time shifts.

The successor must preserve attacks and tonal coherence without combining
independently phased output streams.

## 2) External Evidence Summary

- Rubber Band's public R2 notes describe one block phase-vocoder path with
  transient phase resets, adaptive stretch between resets, and vertical phase
  treatment. Its integration notes say transient placement can make local rate
  differ from the requested long-term average.
- Roebel places transient detection and peak-selective preservation inside the
  phase-vocoder representation. The attack problem is invalid stationary-frame
  phase prediction, not overlap-add by itself.
- Barry, Dorran, and Coyle keep local stretch at unity across every overlapping
  transient frame, then compensate in later steady frames. They also identify
  dense transient sequences as a required conflict case.
- Nonstationary Gabor and multi-scale work use short time resolution around
  attacks and longer frequency resolution for stable components inside one
  reconstructable time-frequency system.
- Duxbury, Davies, and Sandler separate transient/noise bins from steady bins
  with multiresolution analysis and adaptive thresholds. Their results support
  separation as a viable algorithm family, but also expose fixed-resolution,
  threshold, synthetic-component, and spectral-subtraction risks.
- FitzGerald separates harmonic and percussive structures with horizontal and
  vertical median filters plus complementary spectrogram masks. Soft masks
  reduce resynthesis artifacts but increase component interference.
- Driedger, Müller, and Ewert combine harmonic/percussive separation with
  long-frame phase-vocoder TSM and very-short-frame OLA. Their listening study
  found the hybrid competitive with a commercial reference overall, but poor
  separation of singing voice leaked harmonic energy into OLA.
- Driedger, Müller, and Disch address ambiguous and leaking material with a
  third residual component and a separation factor. Their refined procedure
  extracts harmonic content at long resolution and percussive content from the
  complement at short resolution while preserving an exact additive residual.

## 3) Recommendation

Replace full-band branch switching with complementary additive components on
one monotonic time map.

1. Prove iterative harmonic/residual/percussive decomposition before any new
   TSM render. Use long-resolution tightened harmonic extraction, then
   short-resolution tightened percussive extraction from the complement. The
   residual owns ambiguous material.
2. Require binary masks to be disjoint and exhaustive. Reconstructed harmonic,
   residual, and percussive components must sum to the source within fixed
   numerical tolerances.
3. After separation passes, process harmonic content with long-window
   identity-locked phase vocoding, residual content with the current kernel,
   and percussive content with very-short normalized OLA. Give every processor
   the same ratio and target length, then add the components sample-aligned.
4. Derive linked stereo from shared masks, time maps, and component frame
   positions. Retain channel-specific complex spectra and phase propagation.

## 4) Accepted Tradeoffs

- higher offline CPU and memory for two-stage median-filtered analysis
- report-only intermediate proofs that each close one mechanism but do not
  claim full quality promotion
- a new synthesis core rather than further patching of independent output
  branches

## 5) Required Truth Before Adoption

- exact mask partition and source reconstruction before component TSM
- exact component and final output lengths
- explicit component leakage, energy, transient-replica, and recombination data
- no branch switching, component gain matching, or hidden tail envelopes
- full mono gates before linked stereo
- independent listening before Rubber Band-class claims

## 6) Required Prototype Work

- iterative H/R/P separation and source-reconstruction proof
- additive H/R/P fixed-ratio mono gate
- shared-decision linked-stereo gate

## 7) Promotion Target

- `architecture work`
- `contract 082`
- `g10.029` roadmap recompilation

## 8) Sources

| Source | Confidence | Notes |
| --- | --- | --- |
| [Rubber Band technical notes](https://breakfastquay.com/rubberband/technical.html) | high | Public R2 architecture; R3 explicitly differs |
| [Rubber Band integration notes](https://www.breakfastquay.com/rubberband/integration.html) | high | Local rate and transient placement behavior |
| [Roebel, 2003](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf) | high | Peak-selective transient processing inside a phase vocoder |
| [Barry, Dorran, and Coyle, 2008](https://www.dafx.de/paper-archive/2008/papers/dafx08_19.pdf) | high | Unity-rate transient spans, compensation, dense-event risk |
| [Ottosen and Dörfler, 2017](https://arxiv.org/abs/1612.05156) | high | Adaptive resolution and phase locking in nonstationary Gabor frames |
| [Derrien, 2007](https://www.dafx.de/paper-archive/details/ycBIOtuIpgqXPSFM7I3usQ) | medium | Multi-scale low-frequency and residual/high-frequency treatment |
| [Duxbury, Davies, and Sandler, 2001](https://www.dafx.de/paper-archive/2001/papers/duxbury.pdf) | high | Multiresolution transient/steady separation and its threshold/recombination costs |
| [FitzGerald, 2010](https://www.dafx.de/paper-archive/2010/DAFx10/DerryFitzGerald_DAFx10_P15.pdf) | high | Median-filter H/P separation and complementary mask families |
| [Driedger, Müller, and Ewert, 2014](https://qmro.qmul.ac.uk/xmlui/bitstream/123456789/12184/2/Driedger%20Improving%20Time-Scale%20Modification%20of%20Music%20Signals%20Using%20Harmonic-Percussive%20Separation%202013%20Accepted.pdf) | high | Long-PV plus short-OLA TSM, listening results, and vocal leakage failure |
| [Driedger, Müller, and Disch, 2014](https://www.audiolabs-erlangen.de/resources/2014-ISMIR-ExtHPSep/2014_DriedgerMuellerDisch_ExtensionsHPSeparation_ISMIR.pdf) | high | Tightened iterative H/R/P decomposition and exact residual complement |

## Clean-Room Boundary

Do not inspect or translate Rubber Band source code, reproduce unpublished R3
details, or use Elastique internals. External tools remain comparators. Signal's
algorithm is derived from public papers, public technical descriptions, and its
own measured failures.

## First Prototype Outcome

The current-grid local-ratio-one timeline was rejected. It preserved exact
declared anchors but sparse protection and steady-interval compensation moved
unprotected events, produced hops up to `1664` frames, and passed only `9/60`
combined rows. This removes adaptive local time redistribution from the
recommendation. The one-reconstruction-timeline rule remains.

## Transient Ownership Reassessment

The next candidate is fixed-map peak-selective phase reinitialization, not
explicit transient/residual separation.

Roebel's mechanism matches Signal's measured failure and current kernel seam:
transient phase prediction is corrected at spectral-peak resolution while
stationary neighbours remain under ordinary propagation. A time-ramped window
provides the group-delay estimate; peak-local energy position determines the
single centre-adjacent reset frame. Signal already owns complex analysis
spectra, peak tracking, peak-region locking, and per-bin propagation.

Separation is a larger second mechanism. A credible proof would first need a
perfect-reconstruction multiresolution split, soft or adaptive mask continuity,
component-specific stretching, and a recombination contract. It is deferred,
not disproved. Reopen it only if fixed-map peak processing fails the frozen
crest, placement, integrity, spectrum, and combined gates.

## Fixed-Map Peak Proof Outcome

The fixed-map peak proof is rejected. It produced deterministic exact-length
output and complete overlap-add coverage, but improved anchored `L001` crest by
only `0.040942 dB`, worsened mean measurable event placement by `16.851522`
frames, regressed tonal residual in `21/60` rows, and passed `12/60` combined
rows. Peak-local phase reset without time redistribution does not close
Signal's transient defect on the current grid.

Do not tune the group-delay threshold or broaden reset ownership. The next
research task is to define a reconstructable transient/residual separation
boundary before any component candidate is authorized.

## H/R/P Separation Decision

Use refined iterative harmonic/residual/percussive separation, not a two-way
H/P split. The first long-resolution stage extracts only clearly harmonic bins.
The second short-resolution stage extracts only clearly percussive bins from
the complement. Separation factors `beta_h=2` and `beta_p=2` leave uncertain
content in a residual component instead of forcing voice, noise, or moving
pitch into OLA.

Binary masks are appropriate for the first proof because they make partition
truth explicit: every source bin has exactly one owner and all three masked
spectra sum to the source spectrum. The separator must prove source-domain
reconstruction and synthetic component ownership before specialized TSM opens.

If separation passes, harmonic, residual, and percussive outputs share one
ratio and exact target length. They are additive source components, not
full-band alternatives, so their sample-aligned sum does not reopen the
rejected ownership crossfade.

## Next Task

Implement the frozen projected-field and bounded-heap proof. Do not open audio
synthesis or corpus rendering.

## Frequency-Adaptive Reassessment

Exact lattice removed source-map drift without closing attack placement,
replica, or shape gates. Another fixed STFT parameter or phase-reset variant is
not justified. The next candidate changes time-frequency resolution inside one
invertible transform.

Holighaus et al. give a frequency-adaptive nonstationary Gabor construction
with compact frequency supports, a diagonal frame operator, canonical duals,
and perfect reconstruction under the painless support condition. Its
constant-Q layout provides long low-frequency atoms and short high-frequency
atoms without splitting the source into independently rendered outputs. This
directly targets Signal's combination of subtle long-stretch grain, transient
softness, and occasional transient spikes.

Ottosen and Dörfler show that nonstationary Gabor resolution can improve onset
time resolution and sinusoidal frequency resolution. Their complete TSM is not
Signal's design: it detects attacks, holds their local stretch at unity, and
constructs synthesis windows around relocated onsets. Those policies reopen a
rejected timing mechanism. Only the reconstructable transform family transfers.

The first proof therefore performs identity analysis/synthesis only. It must
establish frame bounds, spectral coverage, canonical-dual reconstruction,
coefficient geometry, common-origin impulse delay, and determinism before a
filter-bank phase-gradient design is authorized. This prevents transform
approximation error or band-dependent delay from being mistaken for a stretch
improvement.

Batch 29.6I passes that boundary. On the mixed `4096`-frame control, frame
bounds stayed within `1.2e-7` of unity, peak reconstruction error was
`1.490116119e-7`, every frequency sample was covered, every band satisfied the
painless support condition, and filter/coefficient/output hashes repeated.
This does not yet establish a valid phase-gradient topology on unequal band
lattices.

## Unequal-Lattice Stop

Prusa and Holighaus extend PGHI to filter banks with controlled frequency
variation, but their discrete method assumes one uniform decimation. They call
nonuniform-decimation heap integration significant future work and describe a
filter-bank time-stretch application only as conceivable. Batch 29.6I's rows
therefore cannot inherit that method directly.

Holighaus et al. later provide a better prerequisite: grid-based wavelet
decimation with aligned coefficient rows, perfect reconstruction, and stable
frame bounds at audio-practical redundancy. Their high-resolution published
configuration uses an analytic Cauchy wavelet, `alpha=900`, `1536` channels,
`16` lowpass completion channels, digital `(0,1)` delays, and redundancy `8`.
Its reported frame-bound ratio is `1.20`.

Signal will first reproduce that transform boundary. Unlike the painless
Batch 29.6I frame, channel delays make the full frame operator non-diagonal, so
the proof must derive and verify the complete canonical dual. Only a passing
common-grid reconstruction proof may reopen phase-gradient design.

Batch 29.6J passes. Frequency-response tightening plus the complete alias-block
canonical dual produced condition ratio `1.025819956`, dual residual
`6.225219e-11`, and RMS reconstruction error `5.520117e-13` on the mixed
control. The next research question is now phase transport, not transform
invertibility.

## Common-Grid Phase Decision

Time-frequency reassignment gives the needed interpretation: the time
derivative of channel phase is local instantaneous frequency and the negative
frequency derivative is local group delay. The frozen channel delay is a known
linear phase term. Estimate instantaneous frequency horizontally, use it to
transport every channel phase to the nominal common-grid time, then estimate
vertical phase differences across aligned channels.

Time stretch does not move the proven synthesis lattice. Output column `m`
queries the source coefficient field at exact fractional coordinate
`u=m/ratio`. Magnitude and gradient fields can be interpolated there without
interpolating wrapped phase. Heap integration then operates on a rectangular
output grid with the same canonical-dual synthesis geometry proven by Batch
29.6J.

This is a mechanism hypothesis, not published quality evidence. Batch 29.6K
must prove compensation sign, derivative scale, assignment truth, impulse
placement, symmetry, coverage, and determinism before the corpus opens.

The phase-difference estimator fails before heap integration. Hop `384` leaves
only a `+/-62.5 Hz` unambiguous residual interval. At `8 kHz`, the wavelet
passband is wider and the estimator aliases despite correct delay compensation.
Auxiliary derivative-filter reassignment is the next bounded research question
because it does not depend on wrapped inter-column phase.

## Auxiliary Derivative Decision

Time-frequency reassignment supplies the alias-free alternative. Analyze the
signal through the original filter and a same-position auxiliary filter that
represents its time derivative. The imaginary derivative/original cross-ratio
is local absolute instantaneous frequency. It uses one coefficient column, so
the `384`-frame inter-column phase interval cannot alias it.

Signal derives the auxiliary response from the final tightened filter, not the
untightened mother wavelet. The first proof covers periodic low-to-near-Nyquist
tones, silence, noise, delay compensation, finite ratios, and repeat hashes.
Fractional projection and heap integration remain closed until it passes.

That proof passes with one deterministic maximum-energy carrier estimate per
column. Across `312.5 Hz`, `1 kHz`, `8 kHz`, and `19.5 kHz`, maximum angular
frequency error is `3.614443e-12` radians/sample and maximum compensated
adjacent-channel residual is `8.683081e-10` radians. This closes estimator
selection only; projection, heap integration, synthesis, and corpus rendering
remain separate gates.

The next proof interpolates only magnitude, absolute instantaneous frequency,
and delay-compensated vertical phase derivatives at `u=m/ratio`. It integrates
one output column at a time with a fixed `2*1536` heap cap. This avoids a
whole-render topology whose memory bound grows with duration. Canonical-dual
audio synthesis remains a later proof.

Additional primary source:

| Source | Confidence | Notes |
| --- | --- | --- |
| [Fitz and Fulop, 2009](https://arxiv.org/abs/0903.3080) | high | Derives same-location frequency reassignment from time-derivative auxiliary windows and interprets phase derivatives as instantaneous frequency/group delay |

Additional primary sources:

| Source | Confidence | Notes |
| --- | --- | --- |
| [Holighaus et al., 2012](https://arxiv.org/abs/1210.0084) | high | Frequency-adaptive painless NSG frames, canonical duals, perfect reconstruction, and sliced constant-Q implementation |
| [Ottosen and Dörfler, 2016](https://arxiv.org/abs/1612.05156) | high | Adaptive-resolution PV evidence; onset-local unity stretch is explicitly excluded from Signal's transfer |
| [Prusa and Holighaus, 2022](https://arxiv.org/abs/2202.07498) | medium | Extends phase-gradient reconstruction to controlled-varying filter banks; informs a later mechanism proof, not Batch 29.6I |
| [Holighaus et al., 2023](https://ltfat.org/notes/ltfatnote057.pdf) | high | Uniform grid-based wavelet decimation, deterministic channel delays, perfect reconstruction, and measured frame stability at redundancy `2..8` |

## Full Phase-Gradient Mono Outcome

The frozen whole-band candidate is rejected. It preserved exact mechanism
truth and materially improved tonal texture and direct Rubber Band alignment,
but did not close the defects the operator heard: transient softness and
placement, post-attack replicas, and formant/shape damage remained broad.

This result narrows the next research question. Keep the continuous whole-band
phase-gradient core as evidence, not a product candidate. Investigate only
clean-room mechanisms that preserve attack placement and spectral-envelope
shape without reopening source separation, local time compensation, or
independent output branches.

## Exact-Lattice Reassessment

The first phase-gradient candidate did not realize Signal's requested ratio
inside its analysis lattice. It fixed synthesis hop to `1024`, rounded one
constant analysis hop, then forced exact length only at the output crop. The
resulting lattice ratios were `0.750183`, `1.250305`, and `1.499268`. Across a
five-second source their endpoint mapping errors can reach roughly `40`, `67`,
and `161` frames. The corpus timing failure was `+16.738760` frames on average
and `+137` frames worst-case. That confound must close before a new attack
mechanism is justified.

Public phase-vocoder formulations place analysis frames at centres `C_l` and
synthesis frames at transformed centres `C'_l`; phase propagation uses the
actual difference between adjacent centres. Signal can therefore retain the
published whole-band phase-gradient core while making its source lattice exact:

1. Define absolute analysis position `A_n = round(n * 1024 / ratio)` rather
   than repeating one rounded hop.
2. Use the actual backward and forward integer intervals when estimating each
   centered time-phase derivative.
3. Keep synthesis hop `1024`, requested-ratio frequency integration, heap
   priority, tolerance, window/FFT geometry, padding, normalization, and crop
   policy unchanged.
4. Prove every analysis centre is within `0.5` source frame of the ideal map,
   then run the unchanged complete mono corpus gate.

This is an architectural correction, not a parameter sweep. It can explain
placement drift but is not expected by itself to solve attack softness or
formant shape.

Röbel's shape-invariant phase vocoder is deferred. It is designed for speech
and introduces sinusoidal/noise classification, correlation-based phase
alignment, spectral-envelope estimation, and voiced/unvoiced balance policy.
Röbel's peak-local transient method is also not reopened: Signal already tested
and rejected its group-delay phase-reinitialization family on the fixed map.
Only if exact-lattice phase gradient retains the tonal/comparator gains and
still fails shape may a separately contracted shape-preservation proof open.

## Exact-Lattice Outcome

Exact mapping passed but the candidate remained rejected. It improved `L001`
crest to `2.379387 dB` and tonal regression-free to `57/60`, but timing worsened
`17.789744` frames, replica protection passed `27/48`, and combined remained
`3/60`. Lattice drift was a real confound, not the dominant defect. The next
research decision may now address attack placement and shape directly.

Additional primary sources:

| Source | Confidence | Notes |
| --- | --- | --- |
| [Röbel, 2010](https://www.isca-archive.org/interspeech_2010/robel10_interspeech.html) | high | General analysis/synthesis frame-centre propagation; speech-specific shape-invariant extension |
| [Röbel, 2003](https://www.dafx.de/paper-archive/2003/pdfs/dafx32.pdf) | high | Peak-local transient reinitialization and amplitude-spectrum limits; already tested mechanism family |

## Full Phase-Gradient Reassessment

The additive H/R/P proof invalidates this memo's original component-synthesis
recommendation. Separation itself reconstructed the source exactly, but
independent component TSM damaged timing, integrity, transient replicas, and
static spectrum. More separation tuning would not address that failure.

Three materially different public families were screened:

- WSOLA preserves one locally dominant waveform period. Published reviews
  identify transient skipping/doubling and polyphonic warble, while its common
  transient-preserving variants redistribute local time around attacks. That
  reopens mechanisms Signal has already rejected.
- sinusoidal/residual models remain weakest on broadband noise and attacks and
  again depend on separately synthesized components
- adaptive-resolution and nonstationary-Gabor systems remain credible later,
  but the surveyed TSM method also holds local stretch at unity around detected
  onsets and compensates elsewhere

Prusa and Holighaus provide the next bounded direction. Their phase vocoder
estimates both partial derivatives of STFT phase and integrates the full phase
gradient through a magnitude-prioritized heap. Horizontal and vertical phase
coherence emerge in one whole-band transform without peak tracking, transient
detection, masks, component synthesis, or local time-map compensation. Their
listening test found the method competitive with commercial universal-mode
systems at `1.5x` and `2x` expansion.

Signal will first prove the phase-gradient kernel, not claim product quality.
The proof uses the published fixed-resolution geometry and one global time map.
It must establish deterministic finite derivatives, one phase assignment per
significant bin, conjugate-symmetric reconstruction, exact length, and
synthetic sine/chirp/impulse/two-tone behavior. The complete corpus gate opens
only after that mechanism proof passes.

Known limitations stay visible. The public method stretches rather than
sharpens transients and can alter partial phase relationships in voiced speech.
Adaptive resolution, transient shaping, and voice specialization remain closed
until the unmodified whole-band core has evidence.

Add these sources to the clean-room evidence set:

| Source | Confidence | Notes |
| --- | --- | --- |
| [Prusa and Holighaus, 2022](https://arxiv.org/abs/2202.07382) | high | Full STFT phase-gradient estimation and RTPGHI integration; published listening comparison |
| [Driedger and Muller, 2016](https://www.mdpi.com/2076-3417/6/2/57) | high | TSM review; WSOLA and phase-vocoder artifact boundaries |
| [Roelands and Verhelst, 1993](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html) | high | Original WSOLA family evidence |
| [Balazs et al., 2011](https://arxiv.org/abs/1112.5262) | high | Nonstationary Gabor frame foundations and reconstruction conditions |
