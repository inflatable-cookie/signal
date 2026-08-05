# g10.041 Batch 41.2 - A18 Mechanism Found, Batch 41.1 Retracted

Status: complete
Created: 2026-08-05
Scope: finding `A18`, low-mid pops on transients

## Retraction

Batch 41.1 eliminated the phase-reset hypothesis. That conclusion is wrong and
is retracted. The hypothesis was right all along; the measurement could not see
the artifact.

## Proving The Metric Was Blind

The roadmap's own risk register named this failure mode before the batch ran:
"the metric is blind rather than the artifact absent". Testing it took one
change — inject the artifact and see whether the measurement moves.

Injecting a `pi`-radian phase flip into the sustained `80 Hz` tone, a complete
polarity inversion and unmistakably a pop, moved Batch 41.1's worst-step metric
from `0.02379` to `0.02360`.

The failure is structural, not a threshold that needed adjusting. A pop sits
*on* a transient, and the percussive attack at that instant is a larger low-band
step than the pop is, so worst-step always reports the attack. No threshold
would have helped.

## The Metric That Works

A low-mid pop is a phase discontinuity in the sustained low component, so
measure that: quadrature-demodulate at the carrier, low-pass, take the largest
phase jump.

| condition | carrier phase jump |
| --- | --- |
| unprocessed source, clean | `0.130 rad` |
| unprocessed source, injected `pi` pop | `3.088 rad` |
| stretched `2.0`, transient reset | `2.752 rad` |
| stretched `2.0`, no reset | `0.142 rad` |

It fires on the injected pop by `24x` the floor, so a null from it means
absence. The shipped path measures `2.752 rad` — within `11%` of a deliberate
polarity flip of the low tone.

## The Mechanism

`should_reset_phase_at_transient` sets *every* bin's synthesis phase to the
analysis phase. High bins have short periods, so a phase jump there is a small
time shift. Low bins have long periods, so the same jump is a large waveform
discontinuity — the pop.

Clean material, across ratios:

| ratio | transient reset | no reset |
| --- | --- | --- |
| `0.75` | `0.128` | `0.128` |
| `1.00` | `0.130` | `0.130` |
| `1.25` | `0.164` | `0.143` |
| `1.50` | `0.478` | `0.135` |
| `2.00` | `2.752` | `0.142` |
| `3.00` | `0.595` | `0.251` |

`0.75` and `1.00` match exactly because the reset only engages at ratio `1.0`
and above. The artifact appears from `1.25`, peaks near `2.0` at `19x` the noise
floor, and the no-reset column never leaves the floor.

## It Corroborates The Listening, Not Just Fits It

`D2`, the case where the pop was reported, was static ratio `2.0` — exactly
where the artifact peaks. `D3` ramped to `1.75`, between the `1.5` and `2.0`
rows. The prediction and the report were made independently and agree on where
the artifact is worst.

That is stronger than consistency. A mechanism that peaked at ratio `3.0` would
have fit the reports equally loosely and been wrong.

## Guard

`tests/a18_transient_pop_guard.rs`:

- `a18_metric_detects_an_injected_pop` runs always. It asserts the clean source
  sits near the floor and an injected `pi` pop registers above `2.5 rad`. If the
  metric ever goes blind again, this fails rather than quietly returning nulls.
- `a18_offline_stretch_does_not_break_low_carrier_phase` reproduces `A18` and is
  `#[ignore]`d with the measured value in its reason, following the `g10.039`
  `G5` precedent. Un-ignored it fails at ratio `1.5` with `0.478rad` against a
  `0.130rad` floor.

## The Process Lesson

This is the fourth time this generation a measurement passed while the thing it
measured was broken: `g10.039`'s five structural gates passed a renderer
emitting silence, `g10.040`'s first `G3` passed the shipped quantum-locked
kernel, its first `G7` threshold would have measured frame-grid phase, and now
Batch 41.1's worst-step metric could not see a polarity flip.

The rule that catches all four is the same and costs one extra run: before
believing a null, inject the artifact and check the measurement moves. Batch
41.1 did not, and produced a confident wrong answer that would have closed the
last open finding of the audit incorrectly.

## Next Task

Open Batch 41.3. Reset phase only above a frequency where a phase jump is a
small waveform step. The reset must stay in some form — it exists to stop
transient smearing, and deleting it trades one artifact for another.
