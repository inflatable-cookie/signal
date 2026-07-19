# Offline Time-Stretch Non-Phase-Vocoder Feasibility

Status: complete; no candidate promoted
Owner: dsp
Updated: 2026-07-19
Contract: `084`
Roadmap: `g10.030`, Batch 30.7

## Decision

Close the successor program on the frozen Signal OfflineHighQuality baseline.
No reviewed non-phase-vocoder family justifies a complete clean-room candidate.

This is not a claim that non-phase-vocoder stretch cannot work. It is a
Signal-specific decision against Contract `084`, the frozen comparator pack,
the retained source studies, and the measured failure history.

## Feasibility Gate

A family may open a successor brief only if existing evidence makes all of
these jointly plausible:

- one monotonic sample-domain source/output map
- polyphonic tonal continuity without frame-rate grain or atonal ringing
- exact transient placement without replicas or independently mixed repairs
- one linked-channel law covering every synthesized component
- boundaries, exact length, determinism, and duration-independent working state
- credible improvement over the competitive baseline across the retained
  synthetic and long-form listening families

No family passes.

## Scope Boundary

This gate applies to transparent music and mixed-program replacement across
the production target ratios. It does not reject a separate creative renderer
whose `4x`-`16x` goal is controlled smear, diffusion, and texture rather than
exact transient reconstruction. That path is governed by Contract `085` and
`offline-creative-time-stretch-study.md`; it does not reopen this decision.

## Family Assessment

| Family | Useful property | Blocking evidence | Decision |
| --- | --- | --- | --- |
| WSOLA, PSOLA, source-synchronous overlap-add | bounded waveform-domain synthesis; strong speech behavior | one local lag cannot jointly preserve arbitrary polyphonic periods; expansion copies and compression skips waveform grains; Signal already observed replica and adaptive-timing failures | specialist control only |
| direct subband sinusoidal synthesis | direct oscillators avoid inverse-frame phase loss; octave bands provide simultaneous resolution | pinned SBSMS failed mono integrity, long-form objective position, local stereo consistency, and duplicate/mono-parity/swap/polarity mechanics | closed |
| deterministic sines + transients + stochastic residual | explicit tonal, attack, and noise representations directly match the audible problem | components are analyzed, time-modified, and summed separately; no reviewed law jointly owns partial phase, transient waveform, stochastic covariance, stereo, and recombination | research reserve only |
| neural or learned waveform synthesis | can improve extreme stretching of environmental sounds | reviewed system targets `4x` and `8x`, retains a phase vocoder for sines, processes stereo channels independently, uses probabilistic WaveNet synthesis and beam search, and requires trained model state | outside Signal boundary |

## Why The Sinusoidal Route Does Not Open

The strongest non-phase-vocoder topology is direct subband partial tracking.
It supplies one oscillator clock per matched partial and can keep compatible
channel-relative phase at sample synthesis. Signal already tested the exact
pinned SBSMS topology before funding clean-room work. It repeated, but failed:

- seven mono hard-integrity rows
- two identity rows worse on every quality field than coherent Signal
- `21` five-second development metrics worse than both Signal and Rubber Band
- six local linked-stereo consistency rows
- exact duplicate, mono-parity, channel-swap, and polarity mechanics

Direct oscillators fixed the rejected inverse-frame ownership seam. Track
model error, births, deaths, crossings, unmatched components, and the final
component sum created a different whole-renderer loss. A Signal rewrite would
not have a source-backed reason to beat that measured topology.

## Why Sines + Transients + Noise Does Not Rescue It

The original Verma-Meng model gives each material class appropriate behavior:
sinusoids retain pitch, transients move without changing duration, and noise
stays noise-like. Its published TSM evidence is a mono `22 kHz` drum excerpt at
`1.5x`. It does not define arbitrary polyphonic linked stereo, fixed working
state, exact boundaries, or comparative long-form admission.

More recent fuzzy STN work improves separation and transient preference at
`1.5x` and `2x`, but its renderer remains a phase-vocoder system: tonal and
noise components use phase-vocoder processing, transients are independently
relocated, and the authors identify audible dissonance between preserved
attacks and smeared noisy attack content. It therefore supports better
classification, not a complete non-phase-vocoder successor.

Adding a deterministic stochastic plane to the rejected direct sinusoidal
model would create three independently modified waveform owners. Signal's
earlier H/R/P evidence already showed that exact source decomposition does not
guarantee correct timing, timbre, boundaries, replicas, or stereo after
separate component TSM and recombination.

## Why Neural Synthesis Stays Closed

The reviewed neural STN renderer reports listening wins for `4x` and `8x`
environmental audio. Its target and operating model do not match Signal:

- the requested range is `0.5x` through `2x` music and mixed program audio
- sines still use identity-locked phase-vocoder synthesis
- left and right may be processed independently
- noise uses probabilistic autoregressive synthesis and beam search
- training used a separate corpus, GPU work, and persistent model state

That is neither the requested non-phase-vocoder escape nor the bounded,
deterministic, dependency-free renderer Contract `084` requires.

## Reopen Conditions

Do not reopen this decision for a new coefficient, detector, separator, partial
tracker, stochastic model, or neural prototype. Reassessment requires new
external or operator evidence that changes the whole-system bet:

- a public complete renderer at the target ratios materially beats the frozen
  baseline on comparable polyphonic, transient, boundary, and linked-stereo
  material
- its architecture exposes one joint linked-channel synthesis law across every
  tonal, transient, and residual output owner
- deterministic duration-independent working state and exact-length behavior
  are credible before Signal implementation
- the operator finds the frozen baseline no longer competitive on the retained
  long-form pack

Until then, OfflineHighQuality remains the Signal-owned production baseline.
Contract `084` is closed without promotion. `g10.030` is complete.

## Sources

- [Signal SBSMS source dossier](../research/specimen-dossiers/sbsms-source-architecture.md)
- [Signal whole-family decision](../research/translation-memos/017-whole-family-waveform-ownership-decision.md)
- [Signal waveform-domain source result](../research/translation-memos/018-waveform-domain-linked-stereo-re-entry.md)
- [Verma and Meng, Sines + Transients + Noise TSM](https://dafx.de/paper-archive/details/pmF6ZgSLbsx9SOayuwZq5g)
- [Jang and Park, Multiresolution Sinusoidal TSM](https://doi.org/10.1109/TSA.2004.841048)
- [Roelands and Verhelst, WSOLA](https://www.isca-archive.org/eurospeech_1993/roelands93_eurospeech.html)
- [Fierro and Välimäki, Enhanced Fuzzy STN Decomposition](https://arxiv.org/abs/2210.14041)
- [Fierro et al., Extreme TSM Using Neural Synthesis](https://arxiv.org/abs/2211.16992)
- [SBSMS project architecture](https://sbsms.sourceforge.net/)

## Next Task

None in the OfflineHighQuality successor lane. Retain the baseline and reopen
only when a listed whole-system trigger exists. Creative work proceeds
separately through `g10.031`.
