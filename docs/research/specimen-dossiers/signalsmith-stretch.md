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

State-lineage attribution removes that ambiguity. A target-magnitude phase
oracle driven only by prior horizontal state is substantially cleaner, but all
four isolated tones still retain a frame-rate sideband above `-60 dB`.
Vertical-state feedback is not required. Since horizontal transport is only
one half of the specimen's phase field, the next evidence measures the pinned
complete upstream engine rather than changing another translated equation.

The pinned complete engine confirms that the frozen `-60 dB` ceiling was not a
valid fidelity target for this topology at `2x`. Its four isolated tones
measure `-44.686281` to `-46.016214 dB`; the chord measures `-40.016259 dB`.
Signal is still `9.779 dB` worse on the chord and worse on three tones under
identical quantized input. Source-relative parity, not absolute silence of the
known frame-rate sideband, is the next translation gate.

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
- vertical-state feedback is excluded as a necessary sideband cause
- pinned complete-engine performance rejects the absolute `-60 dB` fidelity
  ceiling
- Signal remains worse than pinned source on three tones and the chord
- exact-input parity is frozen at no more than `1 dB` worse per control
- pinned fractional frequency lookup zero-extends out-of-range bins; Signal
  clamps ten vertical observations per `2x` frame to an edge bin
- a controlled zero-extension variant changes leakage by at most `0.068380 dB`
  and leaves all four paired failures unchanged
- the aligned source trace pins Signalsmith Linear revision `56686735` and
  exposes a `1024`-point modified half-bin transform behind the nominal
  `960/240` support; Signal instead uses a `960`-point standard real transform
- current, reconstructed preliminary, and corrected hashes repeat at aligned
  source centre `8400`; downstream bin deltas are diagnostic because the two
  bases are not isomorphic
- a Signal half-bin-grid-only variant is identity-safe but worsens paired
  parity from `[3, 1]` to `[4, 1]`; the representation is not sufficient alone
- Linear's even-length Kaiser at bandwidth `4` is periodic, not endpoint-
  symmetric; analysis and synthesis coefficients match exactly and are forced
  to exact `960/240` overlap reconstruction
- the window-only Signal variant is identity-safe but improves only two tones;
  paired parity worsens from `[3, 1]` to `[4, 1]`
- the exact periodic-Kaiser/modified-half-bin combination closes paired parity
  to `[0, 0]`; every tone is within `0.147 dB` and the chord is `0.641 dB`
  better than pinned source
- the two analysis choices are one coupled phase-basis representation; neither
  is independently promotable
- the coherent representation passes Signal's full synthetic predictor proof:
  one-frame transient placement, zero replicas, exact silence, all mechanism
  paths exercised, and stable complete hash `0905a7fd4180bff4`
- at `44.1 kHz`, exact source rules derive `5292/1323` support/interval and a
  `6144`-point, `3072`-band modified half-bin transform
- across six exact shared musical inputs, coherent Signal improves event timing
  and static residual on four rows each, replica ratio on three, and boundary
  growth on none; both engines pass hard integrity
- the final two-way pack contains six references and twelve repeat-stable,
  level-matched concealed trials; identity remains closed pending row-complete
  listening

## Next Task

Complete the six-row concealed comparison. Judge boundary behavior explicitly.
Stop before stereo or production selection.
