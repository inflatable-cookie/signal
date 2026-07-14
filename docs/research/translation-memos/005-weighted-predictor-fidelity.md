# Weighted Predictor Fidelity

Status: promoted
Memo: `g10.029` weighted-predictor correction
Owner: dsp
Last updated: 2026-07-14
Related contract: `082`, Rule 31G

## Problem

The first Signal weighted predictor improves on current Signal in four of six
long-form rows, but mutates a bass tone and causes severe sustained-pad phase
damage. It was described as a Signalsmith-style control, yet source reinspection
shows it retained only the broad idea of multi-direction phase evidence.

Local repair would tune a different topology. The scheduling, transform
geometry, phase-gradient observation, normalization, fallback, and update graph
must be corrected together.

## Observed Specimen Topology

Pinned Signalsmith Stretch `1.3.2` configures:

- block support from 120 ms
- a default output interval from one quarter of that support, corresponding to
  30 ms
- a fast real FFT at least as long as the block support
- an approximate confined-Gaussian perfect-reconstruction window
- fixed output intervals with input intervals selected by the local time map
- preliminary horizontal phase transport from an auxiliary spectrum one fixed
  output interval behind the current input centre
- a second low-to-high frequency pass using predictions from one-bin and one
  transform/interval-distance neighbours in both directions
- input-frequency twists sampled at offsets scaled by local time factor
- target-energy normalization with input fallback when combined evidence is
  too weak

At 44.1 kHz, default block support and interval are `5292` and `1323` samples.
The specimen's fast-size rule selects FFT length `6144`, making its rounded long
vertical distance five bins. These are observed specimen values, not Signal
constants.

## Signal Topology

Signal adopts the invariants, not the upstream implementation.

### Geometry and schedule

- output interval `H = max(1, round(sample_rate * 0.03))`
- centered analysis/synthesis support `W = 4H`
- transform length `N = W`; RustFFT may execute arbitrary `N`
- centered square-root Hann analysis and synthesis windows
- exact overlap-operator normalization at every output sample
- fixed output centres separated by `H`
- fixed-ratio input centre `round(output_center / ratio)`
- actual signed input hop measured between adjacent rounded centres
- centered reflection outside the source and exact target-length crop

This keeps the 120/30 ms and fourfold-overlap invariants without importing the
specimen's FFT planner or window code.

### Horizontal prediction

For each bin, carry the previous output complex value. Analyse one auxiliary
input spectrum exactly `H` source samples behind the current input centre.
Predict preliminary output from the previous output and the complex product of
current input with conjugated auxiliary input. Divide by the larger of previous
and current input energy plus the weak-evidence floor. Do not target-normalize
this preliminary state; target-energy normalization belongs to the later
vertical result. Identity remains a direct bypass. The actual rounded inter-
centre input hop does not set this observation distance; it sets the local time
factor used below.

### Vertical re-prediction

Let local time factor `R = H / max(1, actual_input_hop)`. Use distances:

- short output distance `d = 1`
- long output distance `d = round(N / H)`

For a lower output neighbour, observe the input-frequency twist across `dR`
bins toward lower frequency. For an upper output neighbour, observe the inverse
twist across `dR` bins from that neighbour. Fractionally interpolate complex
input values at non-integer bin positions. Combine valid short and long
predictions from both directions.

Process bins in ascending frequency order. Lower neighbours therefore carry
already corrected output state; upper neighbours carry preliminary horizontal
state. DC and Nyquist remain real.

Normalize the combined complex prediction to target input energy. If its norm
is at or below an energy-relative floor, use the current input complex phase at
target energy. Do not include the target horizontal estimate as a fifth vote.

### Closed behavior

- no random vertical distance or phase diffusion above `2x`
- no frequency-partitioned scales
- no peak-region owner replacement
- no window, interval, distance, weight, floor, or update-order sweep
- no copied upstream control flow or FFT/window implementation

## Synthetic Gate

The complete implementation must pass before real-source rendering:

- identity: bit-exact mono bypass
- ratios: exact finite deterministic output at `0.75x`, `1.25x`, `1.5x`, and
  `2.0x`
- coverage: every target sample has positive overlap normalization
- bass: sequential `55`, `82.4069`, and `110` Hz notes retain dominant
  frequency within `0.5` Hz and introduce no octave selection
- chord/pad: steady `110`, `164.8138`, `220`, and `329.6276` Hz components keep
  each dominant peak within `0.5` Hz; out-of-band energy stays below `-60 dB`
- transient: isolated and dense attacks stay within `256` frames of projected
  position and no intermediate replica exceeds either protected attack
- weak evidence: silence remains exact zero; a cancellation control exercises
  fallback without non-finite output
- boundaries: finite first/last samples, no uncovered crop, and no tail created
  by post-render zero fill
- mechanism: short/long, lower/upper, horizontal, corrected, and fallback counts
  are all non-zero across the complete control set
- repeat: evidence and output hashes repeat exactly

These thresholds detect the heard bass mutation and sustained phase damage.
They are not corpus-fitted quality proxies.

## Promotion

Promoted into:

- `docs/architecture/offline-time-stretch-synthesis.md`
- contract `082`, Rule 31G
- roadmap `g10.029`, Batches 29.6CL and 29.6CM

## Result

Batch 29.6CM implemented the complete topology in a release-test-only module.
Exact length, finiteness, coverage, boundaries, repeat, bass, chord peak,
transient, silence, fallback, and mechanism gates passed. The steady four-tone
control produced `-30.200611 dB` out-of-band energy against the frozen
`-60 dB` limit; the unprocessed control measured about `-80.43 dB` under the
same analysis. The implementation is rejected before real-source rendering.

Trace-only attribution assigns the earliest failure to preliminary horizontal
transport. Horizontal-only output measures `-28.182097 dB`; its strongest spur
is one `33.333333 Hz` output frame rate from the nearest tone. An exact overlap
oracle remains clean at `-80.392196 dB`; normalization and significant fallback
are excluded. Vertical correction reduces total leakage slightly but retains
the same frame-rate spur.

Isolated-versus-mixed attribution then rejects mixture as a necessary cause.
Every isolated tone produces a frame-rate sideband above `-60 dB`, even though
nearest-bin auxiliary-ratio variance remains at or below `1.710e-7`. Pinned-
source reinspection finds a translation error in Signal's proof: it normalized
preliminary horizontal output directly to current energy instead of using the
specimen's previous/current energy denominator. That preliminary amplitude
weights the vertical phase sum before final target normalization.

Batch 29.6CP restores that source-faithful denominator and its fixed
weak-evidence floor. It does not close the gate. Complete leakage changes only
to `-30.236852 dB`; the horizontal trace improves to `-29.975234 dB`; all four
isolated tones remain above the ceiling at `-23.586788` to `-51.511127 dB`.
The correction remains part of the translated topology, but it is not the
modulation owner.

The attribution trace has one unresolved state-lineage ambiguity. Its
horizontal spectrum is synthesized before current-frame vertical correction,
but its recurrence begins from the previous frame's vertically corrected
output. Separate direct horizontal recurrence from corrected-state feedback
before changing another mechanism.

## Sources

| Source | Revision | Use |
| --- | --- | --- |
| [Signalsmith Stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | `57b93f4e` | scheduling and prediction topology |
| [Signalsmith Linear STFT](https://github.com/Signalsmith-Audio/linear/blob/7f53cdd1ccd52b409dacf2af24e7ff838c5580cd/stft.h) | `7f53cdd1` | observed window, overlap, FFT sizing, and normalization |

## Next Task

Split direct horizontal recurrence from prior vertically corrected state in
report-only evidence. Keep real-source rendering and parameter changes closed.
