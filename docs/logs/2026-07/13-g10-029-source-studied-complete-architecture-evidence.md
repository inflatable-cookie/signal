# g10.029 Source-Studied Complete Architecture Evidence

Date: 2026-07-13
Batch: 29.6CH
Result: operator listening ready

## Implemented Proof

The report-only Signal candidate runs synchronized `1024`, `2048`, and `4096`
transforms. The long scale exclusively owns low frequencies, the middle scale
owns the middle, and the short scale owns the high band. A full-band middle
spectrum moves both crossovers toward bounded local magnitude valleys and
detects attack state. Phase processing exposes ordinary, peak-locked, reset,
unlocked, attack, and linked-channel counts.

The control runs one `2048` transform. Horizontal instantaneous-frequency
advance is combined with magnitude-weighted vertical predictions from both
frequency directions at one- and four-bin distances. It shares the candidate's
schedule, boundaries, target length, and measurements.

Neither path is reachable from production routing or cache identity.

## External Comparators

- current Signal
- Rubber Band R3 from the frozen development render set
- Signalsmith Stretch `1.3.2`, upstream revision
  `57b93f4e9206a089a45387eaa39bdc9f310d3308`
- Signalsmith Linear helper revision
  `7f53cdd1ccd52b409dacf2af24e7ff838c5580cd`

Signalsmith was built and rendered outside the Signal source tree. Signal has
no new external library dependency.

## Synthetic Evidence

Both Signal architectures pass:

- identity bypass
- exact target length
- complete output coverage
- finite output
- finite boundaries
- exclusive frequency ownership where applicable
- repeated output hash
- transient placement within `256` frames

The fixed-grid weighted predictor passes its complete tone/event quality row:
`0 Hz` tone error and `229` frames maximum event error.

The frequency-partitioned path measures `190` frames maximum event error but
`3 Hz` tone error against the frozen `2 Hz` cap. This is one quality failure,
not a structural or integrity failure. No crossover, window, state, or phase
parameter changes follow.

Architecture output hashes:

- frequency partitioned: `11782ecfa04f8ccf`
- fixed-grid weighted predictor: `606ac2b9c259c97f`

## Development Evidence

Both architectures render all nine frozen mono development rows with zero
length, coverage, non-finite, boundary, or frequency-owner failures. Holdout
reads remain zero.

Aggregate five-field development measurements are:

| Path | Crest | Zero crossing | Derivative | Endpoint | Residual |
| --- | ---: | ---: | ---: | ---: | ---: |
| frequency partitioned | `0.411523` | `0.003880` | `0.002027` | `0.066777` | `0.171027` |
| weighted predictor | `0.446298` | `0.002431` | `0.006353` | `0.057069` | `0.162810` |
| current Signal | `0.491488` | `0.008814` | `0.003368` | `0.054587` | `0.167216` |
| Rubber Band R3 | `0.606078` | `0.006625` | `0.023805` | `0.046840` | `0.144569` |
| Signalsmith Stretch | `0.402838` | `0.004812` | `0.002821` | `0.069368` | `0.170182` |

Lower is better for every field. These aggregates do not decide sound quality;
Rubber Band's known audible advantage is not represented consistently by them.
They remain regression and attribution evidence only.

## Listening Gate

The concealed pack is at:

`target/stretch-source-studied-ch-development-pack`

It contains nine references and `45` blinded trials. Each row compares the two
Signal architectures, current Signal, Rubber Band R3, and Signalsmith Stretch.
The `54` audio files pass length and finiteness checks. Pack hashes are:

- assignment identity: `875dd80994c43efd`
- level gains: `67a955adff0bfc7e`
- notes manifest: `6cfcb102460045a8`

## Next Task

Complete the concealed nine-row listen without opening the key. Then decide at
Batch 29.6CI whether the source-studied architecture earns continuation as a
whole. Do not open a parameter or per-metric repair chain from the `3 Hz` miss.
