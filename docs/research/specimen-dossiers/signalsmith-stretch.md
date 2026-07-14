# Signalsmith Stretch

Status: reviewed; Signal fidelity gap identified
Specimen: Signalsmith Stretch `1.3.2`
Owner: dsp
Last updated: 2026-07-14
Scope: time-stretch phase topology at revision `57b93f4e9206a089a45387eaa39bdc9f310d3308`

## Why This Specimen Matters

Signalsmith Stretch is a compact MIT-licensed polyphonic stretcher with a
published design explanation and a readable implementation. It is the best
open control for checking whether Signal's hard peak-region ownership is a
necessary phase model.

## Defining Bets

- one fixed STFT, not time-adaptive resolution
- default analysis length `0.12` seconds and output interval `0.03` seconds
- the caller supplies input and output block lengths; their ratio defines the
  local time map
- a preliminary horizontal phase-vocoder prediction carries phase through time
- preliminary horizontal energy uses the larger of previous and current input
  energy; target-energy normalization occurs after vertical re-prediction
- a second pass adds short and long predictions from both frequency directions
- complex predictions sum before magnitude normalization, so stronger local
  observations carry more phase weight
- the highest-energy channel supplies the phase decision for each bin; other
  channels retain their current input phase relation to that reference
- pure time stretch uses an identity frequency map; spectral-peak mapping is a
  pitch-shift mechanism, not the time-stretch owner

## Strengths

- one coherent coefficient field; no independently stretched full-band layers
- vertical and horizontal phase evidence are combined instead of selecting one
  hard owner for an entire region
- interchannel phase is explicit in the synthesis step
- input analysis is re-established at one fixed interval even when the caller's
  local time ratio changes
- the implementation and design article agree closely enough to make the source
  a useful architectural specimen

## Weaknesses

- no complete transient model
- above `2x` stretch, vertical observation distance is randomized to trade
  distinct time aliases for diffuse smear; the author identifies this as a hack
- the fixed long window trades attack precision for tonal stability
- exact start/end handling is operationally non-trivial and includes reversed,
  polarity-inverted tail cancellation

## Signal Lessons

### Adopt Carefully

- retain the two-stage idea: horizontal prediction first, then vertical
  correction from more than one direction and distance
- use complex weighted evidence rather than hard nearest-owner replacement as
  the single-grid control
- make linked-channel phase ownership part of the phase algorithm, not a later
  stereo wrapper

### Reject Early

- do not treat the `>2x` randomization fallback as a pro-quality solution
- do not assume peak mapping is required for pure time stretch
- do not port the header as Signal's production engine; it is a comparator and
  control architecture

### Signal Proof Divergence

The first Signal control validated weighted prediction but did not reproduce
the specimen's defining mechanics:

- Signal used window/hop `2048/128`; the default specimen configuration scales
  block/interval to 120/30 ms (`5292/1323` samples at 44.1 kHz before the
  implementation's fast-FFT sizing)
- Signal used same-frame target/neighbour analysis-phase differences; the
  specimen samples input-frequency offsets scaled by local time factor and
  applies those twists to neighbouring preliminary output states
- Signal seeded the final sum with its horizontal prediction and added
  square-root magnitude-weighted neighbours; the specimen performs a separate
  vertical re-prediction, normalizes to prediction energy, and falls back to
  input phase when the combined evidence is weak
- the specimen's update order is part of the prediction graph; Signal computed
  one independent static sum per bin

Long-form evidence matches this gap. Signal's control reduces current-engine
grain in four of six rows but mutates one bass tone and produces severe pad
phase damage. These are mechanism failures, not evidence for a distance or
weight sweep.

The corrected Signal translation keeps the specimen's scheduling and predictor
invariants but not its implementation choices: fixed 30 ms output interval,
fourfold centered support, ratio-projected input centres, fixed-interval
auxiliary horizontal transport, actual-hop time-factor-scaled fractional
input-frequency twists, causal low-to-
high correction, target-energy normalization, and weak-evidence fallback.
Signal retains square-root Hann windows, exact overlap normalization, RustFFT,
and deterministic behavior through `2x`.

Synthetic attribution found one remaining translation error. Signal target-
normalized the preliminary horizontal product. The specimen instead divides by
the larger of previous and current input energy, leaving target normalization
to the vertical result. Isolated tones show the defect without mixture
interference, so this equation is corrected before observation geometry.

Correcting that law preserves source fidelity but does not remove the
frame-rate sidebands. Complete leakage remains `-30.236852 dB`; all isolated
tones still fail. The current horizontal trace starts each frame from the prior
vertically corrected output, leaving direct horizontal recurrence and vertical
feedback conflated.

## Source Inventory

| Source | Type | Revision | Confidence | Notes |
| --- | --- | --- | --- | --- |
| [implementation](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | MIT source | `57b93f4e` | high | fixed STFT, scheduling, phase predictions, channel locking |
| [design article](https://signalsmith-audio.co.uk/writing/2023/stretch-design/) | author explanation | 2023 | high | intended phase model and known time-alias limitation |
| [project page](https://signalsmith-audio.co.uk/code/stretch/) | project documentation | current | high | presets, latency, API, licence |

## Resolved Evidence

- weighted prediction beats current Signal on four of six long-form rows
- the short-window simplified control does not isolate long-window contribution
- frequency-partitioned multi-scale synthesis is rejected independently
- the first combined proof fails frame-rate sidebands before real-source audio
- preliminary horizontal energy scaling is restored for source fidelity
- the correction does not own the sideband failure
- horizontal recurrence versus corrected-state feedback remains unresolved

## Next Task

Split horizontal recurrence from corrected-state feedback in report-only
evidence. Stop before corpus rendering or parameter changes.
