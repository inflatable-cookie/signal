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
- an even-length periodic Kaiser perfect-reconstruction window; the initial
  confined-Gaussian configuration is overwritten
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
  each dominant peak within `0.5` Hz; record `-60 dB` as an absolute diagnostic
  and require Signal out-of-band energy no more than `1 dB` worse than pinned
  source for every isolated tone and the chord
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

Batch 29.6CQ resolves that ambiguity without changing the candidate. A phase
oracle carries prior horizontal state and synthesizes its phase at current
input magnitude, removing the raw recurrence's startup-magnitude memory. It is
cleaner than corrected-state feedback for every isolated tone, but still fails
at `-41.444546` to `-52.739473 dB`; each strongest spur stays within `0.222 Hz`
of one output frame rate. Mixed leakage improves from `-29.975234` to
`-41.558047 dB`. Vertical feedback is not necessary for the modulation.

This does not prove the horizontal equation is wrong. Independent-bin
horizontal transport is an intentionally incomplete field which the vertical
pass is meant to re-lock. Intermediate-output failure cannot choose another
translation change. Measure the pinned upstream complete engine under the same
final-output gate first.

Batch 29.6CR shows the gate, not only Signal, was wrong for fidelity. Pinned
Signalsmith Stretch revision `57b93f4e` uses the same `960/240` default geometry
at `8 kHz` and produces isolated-tone leakage from `-44.686281` to
`-46.016214 dB`; its chord measures `-40.016259 dB`. All isolated strongest
spurs remain one `33.333333 Hz` output frame rate from the tone. The engine is
exact-length, finite, pitch-correct, and bit-repeating at decoded 16-bit output.

Signal remains meaningfully different under the same quantized controls. Three
tones are `8.041` to `21.143 dB` worse, one is `6.225 dB` better, and the chord
is `9.779 dB` worse. Preserve `-60 dB` as an absolute diagnostic, but replace it
as the topology-fidelity rejection criterion with paired pinned-source parity.

Batch 29.6CS freezes the paired gate and retains every non-fidelity gate. The
absolute diagnostic reports `[4 tone, 1 chord]` pinned failures; the `1 dB`
source-relative rejection reports `[3 tone, 1 chord]` Signal failures. Report
direction remains translation research.

The first exact internal differential is frequency-boundary lookup. Pinned
source returns zero outside its spectrum; Signal clamps to the nearest edge
bin. The `2x`, `960/240` geometry changes ten vertical observations per frame.
The ascending dependency graph can propagate those low-frequency decisions,
but causality remains unproven until a controlled boundary-policy ablation.

Batch 29.6CT rejects that mechanism. Zero-extension moves isolated-tone
out-of-band energy by `-0.033206` to `+0.005683 dB` relative to clamping and
moves the chord by `-0.068380 dB`. Both variants retain `[3 tone, 1 chord]`
paired failures. Structure, pitch, and repeat pass. The lookup difference is
real but not material to the fidelity gap.

The aligned Batch 29.6CU trace moves the first divergence earlier. Pinned
Signalsmith Linear revision `56686735` represents the `960`-frame support on a
`1024`-point modified real transform with `512` half-bin bands. Signal uses a
standard `960`-point real transform with `481` bins. The first band centres are
`3.90625 Hz` and `0 Hz`; spacing is `7.8125 Hz` and `8.333333 Hz`. Exact stage
hashes repeat at source centre `8400`. Phase and magnitude differences after
that boundary cannot isolate predictor equations because the bases differ.

Batch 29.6CV tests the modified half-bin grid without changing Signal's
square-root Hann window. Identity error is `2.220e-16`, and all structure,
pitch, and repeat checks pass. The grid improves `110 Hz` by `6.071 dB` but
regresses the other tones by `3.171` to `28.993 dB` and the chord by
`3.736 dB`. Paired failures worsen from `[3, 1]` to `[4, 1]`. Reject the grid
alone.

Source inspection leaves one untested analysis differential. Signalsmith
Stretch explicitly selects Linear's periodic Kaiser window after configuring
the STFT. At `960/240`, the bandwidth argument is `4`; Linear then normalizes
each hop residue class for exact sum-of-squares reconstruction. Test that
window alone on Signal's standard grid before any combined representation.

Batch 29.6CW corrects the prior symmetry assumption and rejects the window
alone. The source analysis and synthesis coefficient hashes are both
`cd811c4f82d161be`; maximum endpoint-mirror delta is `0.002532`. Four-hop
overlap is within `8.953e-8` of unity. The variant improves `110 Hz` and
`220 Hz` by `10.078` and `8.823 dB`, but regresses `164.8138 Hz`, `329.6276 Hz`,
and the chord by `5.906`, `30.764`, and `1.821 dB`. Paired failures worsen to
`[4, 1]`.

Both main effects are now measured. The source uses them together, and the
predictor operates on their phase basis. One combined cell completes the
bounded `2x2`; it is not an open-ended compound repair.

Batch 29.6CX confirms that interaction decisively. The combined periodic
Kaiser and modified half-bin grid closes paired failures from `[3, 1]` to
`[0, 0]`, despite either main effect alone worsening them to `[4, 1]`. Tone
deltas against pinned source are `-0.141`, `+0.147`, `+0.122`, and `+0.129 dB`;
the chord delta is `-0.641 dB`. Factorial interaction ranges from `-3.455` to
`-53.403 dB`. The representation is phase-basis coherent and cannot be reduced
to a better window or a better grid in isolation.

Batch 29.6CY carries that basis through the complete frozen system proof.
Source parity remains `[0, 0]`; bass error is `0.000718 Hz`; chord peak error is
`0.007314 Hz`; transient placement is within one frame with zero replicas;
silence is exact. Structure, identity, boundaries, coverage, cancellation,
mechanism exercise, and repeat pass. The combined basis is now the faithful-
predictor research baseline, still isolated from production selection.

Batch 29.6CZ carries the source geometry to `44.1 kHz` without a new tuning
choice. The exact rules produce `5292/1323` support/interval and a
`6144`-point, `3072`-band transform. Six shared musical inputs pass structure,
hard integrity, and repeat. Coherent Signal beats pinned Signalsmith on event
timing and static residual in four rows each, on replica ratio in three, and on
boundary growth in zero. This mixed result opens one concealed comparison; it
does not establish musical superiority.

Batch 29.6DA freezes the authorized comparison as six source references and
twelve level-matched concealed trials. Audio, assignment, gain, manifest, key,
notes, and metadata-receipt hashes repeat. No holdout, stereo, dynamic-ratio,
or product path is present. The representation remains report-only until the
six-row listening record resolves continuity, grain, tone, transients, and
both boundaries.

## Sources

| Source | Revision | Use |
| --- | --- | --- |
| [Signalsmith Stretch](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | `57b93f4e` | scheduling and prediction topology |
| [Signalsmith Linear STFT](https://github.com/Signalsmith-Audio/linear/blob/5668673560146a9cfe38c25315071e3fd68c8317/stft.h) | `56686735` (`0.3.1`) | observed window, overlap, modified FFT sizing, half-bin grid, and normalization |

## Next Task

Complete the hash-frozen concealed comparison and return row-complete findings.
Keep equation changes, third mechanisms, stereo, dynamic ratio, and product
routing closed.
