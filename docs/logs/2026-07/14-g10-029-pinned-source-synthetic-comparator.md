# Pinned-Source Synthetic Comparator

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CR
Scope: exact-input pinned Signalsmith final-output comparison

## Decision

Reject `-60 dB` as the topology-fidelity ceiling at `2x`. Retain it as an
absolute diagnostic. Signal still fails source parity.

The comparator verifies Signalsmith Stretch source revision
`57b93f4e9206a089a45387eaa39bdc9f310d3308`, CLI version `1.3.2`, and default
`8 kHz` geometry `H=240`, `N=960`, fourfold overlap. The CLI binary must reside
inside the verified checkout. It receives the same mono 16-bit controls used
for paired Signal renders and executes its fixed-file seek/process/flush path.

## Evidence

| Control | Input OOB | Pinned OOB | Signal OOB | Signal minus pinned | Pinned sideband | Pinned hash | Signal hash |
| ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| `110 Hz` | `-77.658183 dB` | `-46.016214 dB` | `-26.552889 dB` | `+19.463324 dB` | `33.554688 Hz` | `7069b2be6cef6725` | `ece10d1f7f1115e8` |
| `164.8138 Hz` | `-79.839256 dB` | `-45.800547 dB` | `-37.759764 dB` | `+8.040783 dB` | `33.428388 Hz` | `570edabe6cef6725` | `bea692e61f3a72c5` |
| `220 Hz` | `-78.799064 dB` | `-44.686281 dB` | `-23.543720 dB` | `+21.142561 dB` | `33.417969 Hz` | `a76daebe6cef6725` | `218cb9d30316ce82` |
| `329.6276 Hz` | `-74.776984 dB` | `-45.272110 dB` | `-51.497422 dB` | `-6.225312 dB` | `33.165369 Hz` | `ee8ed9be6cef6725` | `f9667d3eaf80c2a9` |
| chord | `-79.568640 dB` | `-40.016259 dB` | `-30.236975 dB` | `+9.779283 dB` | `27.207031 Hz` | `c4a9f43e6cef6725` | `8d123802d06425c0` |

Pinned tone peak error is `0.002673` to `0.007331 Hz`; chord peak error is
`0.007362 Hz`. All paired outputs have exactly `32000` frames, contain no non-
finite samples, and repeat at the decoded-sample hash boundary.

Every pinned isolated tone has a strongest spur within `0.222 Hz` of the
`33.333333 Hz` output frame rate. The frame-rate sideband is therefore a
property of the studied complete topology at this ratio, not proof by itself
of Signal mistranslation.

## Consequence

The prior gate conflated two questions:

- absolute cleanliness: useful diagnostic, missed by both engines
- translation fidelity: paired source parity, missed materially by Signal on
  three tones and the chord

The next contract uses exact-input parity with a `1 dB` allowance per control.
That target remains strict enough to expose Signal's current `8–21 dB` losses
without demanding a result the pinned source cannot produce.

## Closed Lanes

- predictor, geometry, floor, weight, window, interval, distance, and FFT edits
- corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CS. Replace the absolute fidelity rejection with paired pinned-
source parity while retaining `-60 dB` as a diagnostic. Identify one internal
differential for later implementation; change no DSP in that batch.
