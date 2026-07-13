# Signalsmith Stretch

Status: reviewed
Specimen: Signalsmith Stretch `1.3.2`
Owner: dsp
Last updated: 2026-07-13
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

## Source Inventory

| Source | Type | Revision | Confidence | Notes |
| --- | --- | --- | --- | --- |
| [implementation](https://github.com/Signalsmith-Audio/signalsmith-stretch/blob/57b93f4e9206a089a45387eaa39bdc9f310d3308/signalsmith-stretch.h) | MIT source | `57b93f4e` | high | fixed STFT, scheduling, phase predictions, channel locking |
| [design article](https://signalsmith-audio.co.uk/writing/2023/stretch-design/) | author explanation | 2023 | high | intended phase model and known time-alias limitation |
| [project page](https://signalsmith-audio.co.uk/code/stretch/) | project documentation | current | high | presets, latency, API, licence |

## Open Questions

- How much of its long-stretch quality comes from the long fixed window versus
  multi-predictor phase transport?
- Does it beat current Signal on the frozen bass and sustained development rows?
- Can its weighted predictor remain a useful control inside a frequency-
  partitioned multi-scale system?

## Next Task

Add this engine to the frozen comparator matrix. Use its fixed single-grid
multi-predictor topology as the control for the next complete Signal candidate.
