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

Implement the report-only H/R/P separation and source-reconstruction proof
frozen in contract `082`. Do not implement component TSM until that gate
passes. Keep linked stereo and production routing closed.
