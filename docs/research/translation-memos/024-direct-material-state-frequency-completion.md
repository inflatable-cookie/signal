# Direct Material-State Frequency Completion

Status: promoted for no-audio contract
Date: 2026-07-19
Roadmap: `g10.029`, Batch 29.7BD
Contract: `082`, Rule 31AF

## Question

Does retained AX/BC evidence support another direct mechanism, or should the
direct peak-ownership branch close after channel-local peaks leave its dominant
failure signature unchanged?

## Retained Evidence

AX and BC are frozen. AX remains `397128c177d3033e` at `38/48` calibrated
failures, `157/384` improved windows, `36/48` local failures, and residual
`0.7611955347641768`. BC remains `b13c37cff1b58afa` at `40/48`, `159/384`,
`36/48`, and the same residual. BC changes all `48` candidate hashes, so the
channel-local peak mechanism is active. It changes only four row outcomes:

- two `0.75x`, `16384`-frame, phase-`0.00` image rows cross the calibrated
  mid/side limit in the wrong direction
- two `0.75x`, `16384`-frame, phase-`0.37` image rows gain one improved local
  window
- no row changes local-failure classification
- the worst row remains the `2.00x`, `16384`-frame, phase-`0.37`, comparison-
  aligned tone with residual `0.7611955347641768`

The tone split is stronger than the AX/BC delta. All `24/24` tone rows fail
local consistency. For the `12` comparison-aligned tone rows, candidate
interior IPD spans `1.153146` to `2.343541` radians, zero local windows improve,
and the maximum local residual is `0.761196`. For the `12` comparison-unaligned
rows, interior IPD spans only `0.00000298` to `0.00037522` radians and `39`
local windows improve.

The comparison label describes a `1024`-point grid, not Signal's direct long
scale. At `8 kHz`, comparison-aligned frequency `246.09375 Hz` is direct-bin
position `19.6875`, or `-0.3125` bin from the nearest `640`-point long-scale
bin. Comparison-unaligned `248.984375 Hz` is position `19.91875`, only
`-0.08125` bin away. The catastrophic group is therefore the group farther
from Signal's own synthesis grid.

Both frequencies are below `750 Hz` and use only the long scale. Scale
crossover, scale summation, and channel-local peak disagreement cannot explain
this split. Phase and source-frame variants retain it.

## Code Audit

The direct guidance path computes temporal and three-atom frequency medians,
then stores fuzzy tonalness, noisiness, and transientness independently for
each atom. Terminal state consumes those raw ratios immediately:

1. unsupported history resets
2. a transient winner at a detected centre attacks
3. noisiness above tonalness unlocks
4. everything else locks

There is no material label field, modal frequency completion, or contiguous
material-owned state range. A stationary tone between direct bins spreads
energy across neighbours. That raises the vertical median and derived
noisiness across the lobe, allowing grid position alone to fragment one
stationary structure between `Locked` and `Unlocked` decisions.

The ordinary recurrence is standard and channel-local. Locked processing now
has channel-local peaks and compatible trajectory borrowing. Each scale
inverse-synthesizes once into its own channel. Those paths contain no remaining
parameter-free ownership mismatch with causal reach over the alignment split.

## Source Mismatch

Pinned Rubber Band R3 does not send continuous per-bin classifier ratios
straight to phase state. Its classification spectrum becomes discrete
harmonic, percussive, or residual evidence; a modal frequency filter converts
those labels into bounded frequency ranges; those ranges then control reset,
kick, unlock, and locked processing. The labels guide one synthesis path and
are not additive H/P/R source separation.

Damskagg and Valimaki independently support fuzzy material evidence controlling
different phase laws. The missing seam is not the fuzzy evidence. It is the
frequency-complete ownership layer between evidence and terminal phase state.
No Rubber Band expression, width, range, threshold, or constant transfers.

## Decision

Close channel-local peak ownership as the active correction branch. Retain its
validated mechanics, but open no more peak-map, predecessor, borrowing,
offset, or relation repairs.

One direct mechanism remains admissible: `MaterialStateFrequencyCompletion`.
It must turn shared atom-local evidence into deterministic, contiguous
frequency-owned state ranges before phase processing. This is a topology
change, not a classifier-parameter experiment.

This evidence establishes causal reach, not objective passage. Another audio
candidate remains closed.

## Bounded No-Audio Falsifier

Batch 29.7BE is implementation-free. It must freeze one Signal-owned
label/tie/mode/range construction and a coefficient-only proof before code.
The later proof is limited to the direct `8 kHz`, `640`-point long scale and
the two retained frequencies `246.09375 Hz` and `248.984375 Hz`, with phase
relations `0.00` and `0.37` and analysis advances for `0.75x`, `1.5x`, and
`2.0x`.

After the existing `19`-tick guidance halo and one predecessor priming step,
the proof must report raw guidance, raw winner, completed label, completed
range, terminal state, peak region, and deterministic hash. It must establish:

- current raw state ownership changes with fractional direct-bin position
- one stationary lobe receives one stable material-state range after
  completion rather than isolated atom decisions
- phase relation, channel swap, magnitude, peak identity, scale ownership,
  state precedence, fixed capacity, finiteness, and repeat remain unchanged
- transient and residual analytic controls remain distinguishable; completion
  cannot turn the whole scale into one locked range

Failure closes `MaterialStateFrequencyCompletion` without audio. Passage may
open mechanics implementation only. A later objective candidate requires its
own failure-first preregistration after complete no-audio passage.

## Excluded

- no corpus or renderer execution
- no mode-width, median-span, threshold, crossover, peak-density, or offset
  sweep
- no transferred source expression or constants
- no listening, export, holdout, mono, long-development, production, dynamic-
  ratio, realtime, routing, or Batch 29.8 work

## Sources

- [Rubber Band R3 classifier](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/BinClassifier.h)
- [Rubber Band R3 guide](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/Guide.h)
- [Rubber Band R3 phase advance](https://github.com/breakfastquay/rubberband/blob/v4.0.0/src/finer/PhaseAdvance.h)
- [Damskagg and Valimaki, Audio Time Stretching Using Fuzzy Classification of Spectral Bins](https://doi.org/10.3390/app7121293)
- [Rubber Band source architecture](../specimen-dossiers/rubber-band-source-architecture.md)
- [Direct channel-local peak topology](./023-direct-channel-local-peak-topology.md)

## Next Task

Run Batch 29.7BE under Rule 31AF. Freeze the Signal-owned material label,
modal completion, range, tie, and coefficient-only falsifier contract. Do not
implement it or run audio in that batch.
