# Preliminary Horizontal Energy Law

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CP
Scope: source-faithful equation correction and synthetic rerun

## Decision

Retain the corrected preliminary energy law. Reject it as the sideband cure.

Signal now divides the prior-output/current-input/conjugated-auxiliary product
by the larger of previous and current input energy plus the pinned fixed floor.
Final target-energy normalization remains in vertical re-prediction. No other
frozen mechanism changes.

## Evidence

- complete chord leakage: `-30.236852 dB`, previously `-30.200611 dB`
- horizontal trace leakage: `-29.975234 dB`, previously `-28.182097 dB`
- input control leakage: `-80.429254 dB`
- maximum bass error: `0.004089 Hz`
- maximum chord peak error: `0.024631 Hz`
- maximum transient error: `256` frames
- replica failures: `0`
- silence peak: `0`
- complete evidence hash: `e7cc3f04c24b5d18`

All structural, repeat, pitch, octave, transient, replica, silence, fallback,
and mechanism gates still pass. The steady sideband gate still fails.

| Tone | Isolated OOB | Spur offset | Hash |
| ---: | ---: | ---: | --- |
| `110 Hz` | `-26.719393 dB` | `33.339844 Hz` | `db662ac6cf32fb17` |
| `164.8138 Hz` | `-37.780438 dB` | `33.428388 Hz` | `d2189114bff29738` |
| `220 Hz` | `-23.586788 dB` | `33.476563 Hz` | `24e6eb186e42241a` |
| `329.6276 Hz` | `-51.511127 dB` | `33.165369 Hz` | `33bc2750eb5a6da1` |

Mixed horizontal leakage is `-29.975234 dB`, hash `d5806c0a78122f0d`.
Every isolated tone still fails. The energy correction is not the phase-
modulation owner.

## Attribution Gap

The existing horizontal trace synthesizes current preliminary output but feeds
the next frame from the previous complete, vertically corrected state. It
cannot distinguish direct horizontal recurrence from modulation introduced by
vertical correction and returned through that state.

## Closed Lanes

- floor, weight, window, interval, distance, and FFT sweeps
- corpus, holdout, listening, stereo, and dynamic ratio
- cache and production routing

## Next Task

Run Batch 29.6CQ. Compare prior horizontal and prior corrected recurrence in
report-only evidence. Stop with one mechanism owner before another equation or
geometry change.
