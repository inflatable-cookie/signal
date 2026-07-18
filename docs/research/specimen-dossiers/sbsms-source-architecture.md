# SBSMS Source Architecture

Status: reviewed
Specimen: SBSMS `2.3.0`
Owner: dsp
Last updated: 2026-07-18
Scope: exact source at `e99cd7e6c6367e476577be34d2fdbe2023904d7e`

## Why This Specimen Matters

SBSMS is a complete subband sinusoidal analysis and resynthesis engine. Unlike
Signal's rejected phase-vocoder families, it does not modify a redundant
coefficient field and then recover waveforms through inverse frames, support
restriction, and overlap normalization. It tracks partials through recursive
octave subbands and synthesizes their output waveforms directly.

The source is GPL-2.0. Signal uses it only as architecture evidence and as a
pinned external behavioral comparator. No source expression, constants,
thresholds, tables, or dependency enter Signal.

## Whole-Renderer Topology

The pinned source provides one connected pipeline:

1. recursively split the input into octave subbands
2. extract sinusoidal peaks and continue, split, merge, start, and end tracks
3. stitch corresponding track state across subbands
4. match compatible tracks across stereo channels
5. derive paired frequency and phase evolution jointly
6. synthesize each track with a direct sample-domain oscillator whose
   magnitude and frequency evolve through the segment
7. sum every subband on one output sample timeline

Track births, deaths, jumps, splits, merges, and short high-frequency subbands
own discontinuous and noise-like material inside the same track topology. The
renderer does not add an independently stretched transient or stochastic
residual plane.

## Linked-Stereo Mechanism

The pinned stereo path explicitly pairs track points between channels. Paired
frequency evolution is updated jointly. Phase update is deferred until the
counterpart is available, then the peer synthesis phase is related to the
counterpart through the current analysis-phase difference.

The useful invariant is narrower and stronger than Signal's failed per-bin
constraint:

> Each matched partial has one shared output oscillator clock and trajectory;
> each channel retains its current partial magnitude and analysis-relative
> phase relation at that directly synthesized waveform.

The invariant exists at a waveform-producing component. There is no inverse
STFT projection, finite inverse-frame crop, or overlap normalization between
the linked relation and its samples. Subbands then add those samples on the
same channel-paired output timeline.

This does not prove exact preservation of an arbitrary full-scene Gram matrix.
Unmatched tracks, model error, partial crossings, births, deaths, and the sum
of nearby components can still change reconstructed waveform relations.
Signal's sample-domain IPD, correlation, mid/side, Gram, and local-consistency
gates remain authoritative.

## Strong Evidence

- linked stereo is explicit track ownership, not independent channel render
- direct oscillators avoid the two inverse-synthesis losses found in 29.7AF
- tonal, discontinuous, and noise-like material stay inside one renderer
- recursive subbands provide different time/frequency resolution without a
  separately overlapped frequency-adaptive output field
- fixed input blocks and a finite active-track set can define bounded work

## Weaknesses And Exclusions

- the sinusoidal model is lossy even at identity unless the analysis and track
  representation are sufficiently complete
- transient sharpness, noise texture, boundaries, and long-stretch stability
  remain behavioral questions, not source-reading conclusions
- the reference implementation allocates dynamic track objects and containers;
  it is not an audio-thread implementation model
- a Signal proof needs preallocated track storage, explicit maximum track and
  event counts, deterministic overflow failure, and a duration-independent
  memory bound
- GPL source structure and numeric policy are excluded from clean-room Signal
  implementation

## Signal Translation Boundary

The selected research candidate is `LinkedSubbandSinusoidalModel`:

- one shared output sample schedule across channels and subbands
- a bounded bank of partial identities and predecessor state
- explicit cross-channel matching for compatible partials
- one output oscillator trajectory per matched partial
- current channel magnitude and analysis-relative phase retained at synthesis
- direct sample synthesis and one final subband sum
- explicit starts, ends, tails, and overflow results

This is an architecture candidate, not implementation authorization. The exact
SBSMS specimen must first demonstrate that the source topology itself reaches
the declared Signal objective envelope on frozen synthetic, mono, stereo, and
boundary material. Failure closes the candidate before clean-room work.

## Pinned Source Inventory

| Source | Revision or hash | Use |
| --- | --- | --- |
| [SBSMS repository](https://github.com/claytonotey/libsbsms/tree/e99cd7e6c6367e476577be34d2fdbe2023904d7e) | `2.3.0`, `e99cd7e6c` | complete source and license boundary |
| [project architecture](https://sbsms.sourceforge.net/) | public project description | octave-subband sinusoidal model and resynthesis |
| [`sms.cpp`](https://github.com/claytonotey/libsbsms/blob/e99cd7e6c6367e476577be34d2fdbe2023904d7e/src/sms.cpp) | SHA-256 `b6b371a2314c8723980a47b69a418272eb8c5052da7f0b151e9e1e9d3202fd4f` | peak, track, stereo-match, and phase coordination |
| [`track.cpp`](https://github.com/claytonotey/libsbsms/blob/e99cd7e6c6367e476577be34d2fdbe2023904d7e/src/track.cpp) | SHA-256 `83e3aa29b062ba9ec78c0e56d6bed3a8bfe6022d328f96085a0903084ce19bda` | direct oscillator and paired trajectory synthesis |
| [`subband.cpp`](https://github.com/claytonotey/libsbsms/blob/e99cd7e6c6367e476577be34d2fdbe2023904d7e/src/subband.cpp) | SHA-256 `a803d764d008a756d0bb1cd6e1b25ad3bc015567cadc4ceca09d894a1ad4d896` | recursive subband pipeline and output sum |

## Next Task

Batch 29.7AH may build and run this exact revision only as an external research
specimen under `target/`. Freeze the existing development material first, then
measure source-attainable identity, mono, stereo, boundary, repeat, runtime,
and active-state behavior. Do not copy source or implement a Signal renderer.
