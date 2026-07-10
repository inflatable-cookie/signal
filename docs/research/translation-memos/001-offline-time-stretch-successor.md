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

## 3) Recommendation

Replace waveform-branch switching with one monotonic synthesis timeline.

1. Prove transient ownership first on the current `2048/512` grid. Keep the
   global time map fixed. Use the frozen classifier as an event guard, estimate
   peak-local attack position from a time-ramped companion FFT, and
   reinitialize only collected transient peak regions near the window centre.
2. Add adaptive time-frequency resolution only after that proof passes its
   transient and placement gate. Short and long frames must share one
   nonstationary reconstruction law; they must not produce separate waveforms
   for later crossfade.
3. Keep vertical phase coherence peak-driven and adaptive. Do not reopen scalar
   lock thresholds or copy unpublished Rubber Band methods.
4. Derive linked stereo from the same time map, frame schedule, transient
   resets, and shared peak regions.

## 4) Accepted Tradeoffs

- higher offline CPU and memory for adaptive analysis
- report-only intermediate proofs that each close one mechanism but do not
  claim full quality promotion
- a new synthesis core rather than further patching of independent output
  branches

## 5) Required Truth Before Adoption

- exact output length and source-projected transient anchors
- explicit dense-transient conflict/fallback reporting
- no branch time shifts, waveform crossfades, or hidden tail envelopes
- full mono gates before linked stereo
- independent listening before Rubber Band-class claims

## 6) Required Prototype Work

- current-grid fixed-map peak transient proof
- nonstationary/adaptive-resolution reconstruction proof
- combined fixed-ratio mono gate
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

## Next Task

Reassess contract `082` for explicit transient/residual separation. Freeze its
perfect-reconstruction split, mask continuity, component processing,
recombination, evidence, and stop conditions before implementation. Keep
adaptive resolution, linked stereo, and production routing closed.
