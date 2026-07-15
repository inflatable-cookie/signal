# Rubber Band Source Architecture

Status: reviewed
Specimen: Rubber Band R2 and R3
Owner: dsp
Last updated: 2026-07-15
Scope: source topology at revision `e4296ac80b1170018a110bc326fd0d45a0eb27d6`

## Why This Specimen Matters

Rubber Band remains the audible comparator Signal has not matched. Public
behavioural probes established that local scheduling, event phase, vertical
phase, and R3 standard/short modes matter. Source study now identifies how
those stages are composed.

This dossier records architecture and invariants only. Rubber Band is GPL-2.0-
or-later with a commercial option. No source expression is copied into Signal.

## R2 Architecture

R2 is one block phase vocoder. Its normal path separates:

1. magnitude-derived transient study
2. signed output-increment calculation with exact global closure
3. phase reset selection
4. horizontal phase advance
5. descending-bin lamination
6. inverse transform, windowed overlap, and optional resampling

The default source values use a `2048` FFT and `256` input increment. A reset
leaves selected analysis phases unmodified. Otherwise instantaneous-frequency
error advances phase by the actual output increment. Lamination conditionally
inherits phase advance from a higher neighbouring bin over bounded distances;
it is neither nearest-peak identity locking nor a scalar coherence target.

## R3 Architecture

R3 standard changes the representation, not only the detector:

- one classification FFT establishes a full-band magnitude reference
- long, classification, and short FFT scales run simultaneously
- the long scale owns low frequencies, the classification scale owns the
  middle, and the short scale owns the high band
- crossover frequencies descend to nearby spectral valleys inside bounded
  low and high regions
- larger output hops collapse the high band into the classification scale
- R3 short mode removes this scheme and uses one classification-sized window

The simultaneous scales are not redundant full-band renders. Each scale
resynthesizes only its current frequency interval. Scale outputs then sum on one
output timeline. A shorter synthesis window limits the alias cost of the
frequency-domain cuts.

## R3 Guidance And Phase

The classification spectrum feeds horizontal and vertical median filters.
Bins become harmonic, percussive, or residual. A modal frequency filter turns
those labels into three boundaries: low percussive extent, high percussive
extent, and residual high extent.

Those boundaries guide synthesis rather than create additive components:

- kick and pre-kick ranges preserve low-frequency attack energy
- a residual/percussive gap can open a bounded phase-reset range
- residual high bands can run unlocked
- stretches above `2x` progressively unlock high frequencies and reduce the
  linked-channel range, preferring controlled diffusion over metallic locking
- silence and unity have explicit reset policies

Each FFT scale owns a peak-guided phase state. Ordinary instantaneous-frequency
advance is computed first. Each bin then either resets, remains unlocked, or
inherits a tracked peak advance plus a frequency-band-dependent multiple of
its current analysis-phase offset. Peak density and offset scaling vary by
frequency and ratio. Stereo decisions can borrow the greatest channel's peak
trajectory when both channels are inside the linked range.

## What Signal Previously Got Wrong

- Signal selected one full-band resolution per time centre. R3 selects
  simultaneous resolutions by frequency.
- Signal's rejected union rendered every frequency through every scale. R3
  assigns each output frequency to one scale at a time.
- Signal promoted H/P/R separation as possible additive synthesis. R3 uses the
  labels as control evidence for crossover, reset, unlock, and attack policy.
- Signal treated active peak ownership as one region rule. R3 composes ordinary
  advance, tracked peak advance, scaled local offsets, reset ranges, unlocked
  ranges, and channel ownership.
- Signal inferred R3 standard/short only from output differences even though
  the source exposes their structural distinction.

## Adopt Carefully

- simultaneous frequency-partitioned resolutions on one output timeline
- one full-band classification reference that controls, but does not synthesize
  as, H/P/R components
- dynamic crossovers placed at stable spectral valleys
- separate ordinary, locked, reset, unlocked, kick, and channel-linked states
- complete-system evaluation; no isolated parameter search

## Reject Early

- copying GPL implementation or constants into Signal
- full-band layer recombination
- time-selected resolution as the primary multi-resolution axis
- treating classifier labels as hard additive source separation
- using one peak-region rule for every material state

## Source Inventory

| Source | Type | Revision | Confidence | Notes |
| --- | --- | --- | --- | --- |
| [R3 guide](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/Guide.h) | GPL source | `e4296ac` | high | scale bands, crossover, reset, kick, unlock, channel policy |
| [R3 phase advance](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/PhaseAdvance.h) | GPL source | `e4296ac` | high | peak tracking, offset scaling, channel-linked phase |
| [R3 stretcher](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/R3Stretcher.cpp) | GPL source | `e4296ac` | high | analysis, guidance, per-scale synthesis, output sum |
| [R3 classifier](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/finer/BinClassifier.h) | GPL source | `e4296ac` | high | H/P/R control evidence |
| [R2 process](https://github.com/breakfastquay/rubberband/blob/e4296ac80b1170018a110bc326fd0d45a0eb27d6/src/faster/StretcherProcess.cpp) | GPL source | `e4296ac` | high | reset, advance, lamination, overlap |
| [technical notes](https://breakfastquay.com/rubberband/technical.html) | author documentation | current | high | published R2 summary |

## Open Questions

- Which source-derived invariants explain the existing R3 standard-over-short
  wins on Signal's frozen development rows?
- What Signal-owned crossover and phase-offset policies reproduce the
  architecture without inheriting Rubber Band expression or fitted constants?
- Can one complete frequency-partitioned slice beat current Signal before
  classifier sophistication is widened?

## Exact-Source Baseline Evidence

Batch 29.6DB compares coherent Signal directly with Rubber Band R3 `4.0.0` on
six exact five-second mono inputs at `1.5x` and `2.0x`. Both engines pass hard
integrity and repeat. Coherent Signal has lower static residual on all six rows
and lower timing error on four. Rubber Band has lower replica ratio on five and
lower boundary growth on all six. The objective split does not establish
audible parity or superiority; a peak-safe RMS-matched concealed pack is open.

## Next Task

Complete the Batch 29.6DB six-row concealed comparison. Use the decoded result
to decide whether the coherent single-grid baseline is already competitive or
which Rubber Band source invariant remains audibly material.
