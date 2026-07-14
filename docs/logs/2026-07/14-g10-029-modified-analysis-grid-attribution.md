# Modified Analysis-Grid Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CV
Scope: report-only modified half-bin transform-grid ablation

## Decision

Reject the pinned `1024`-point modified half-bin grid as a standalone owner of
the Signalsmith parity gap. Keep Signal's standard grid frozen.

The variant changes only transform length, band origin, and half-spectrum
packing. It retains the `960`-frame support, `240`-frame interval, square-root
Hann window, schedule, predictor equations, distances, normalization,
fallback, boundary policy, and overlap ownership.

## Structural Evidence

- exact analysis/synthesis identity maximum error: `2.220446e-16`
- output length, coverage, finiteness, and boundary failures: zero
- maximum tone peak error: `0.007531 Hz`
- chord peak error: `0.047758 Hz`
- all output hashes repeat

## Fidelity Evidence

| Control | Pinned | Signal baseline | Half-bin grid | Grid minus baseline | Grid minus pinned |
| --- | ---: | ---: | ---: | ---: | ---: |
| `110 Hz` | `-46.016214 dB` | `-26.552889 dB` | `-32.623549 dB` | `-6.070660 dB` | `+13.392665 dB` |
| `164.8138 Hz` | `-45.800547 dB` | `-37.759764 dB` | `-33.432946 dB` | `+4.326817 dB` | `+12.367601 dB` |
| `220 Hz` | `-44.686281 dB` | `-23.543720 dB` | `-20.373037 dB` | `+3.170682 dB` | `+24.313243 dB` |
| `329.6276 Hz` | `-45.272110 dB` | `-51.497422 dB` | `-22.504905 dB` | `+28.992517 dB` | `+22.767205 dB` |
| chord | `-40.016259 dB` | `-30.236975 dB` | `-26.500523 dB` | `+3.736453 dB` | `+13.515736 dB` |

Hashes:

- `110 Hz`: `ed99b7304bdfdd6b`
- `164.8138 Hz`: `bcedcde4945bad2f`
- `220 Hz`: `2c3cee958e77c777`
- `329.6276 Hz`: `9e90665207d857bd`
- chord: `440880e3f642c797`

Source-relative failures worsen from `[3 tones, 1 chord]` to
`[4 tones, 1 chord]`. Only `110 Hz` improves against Signal baseline. The grid
does not produce coherent parity movement.

## Consequence

Do not promote or combine the half-bin variant. The next isolated observed
differential is the source window: symmetric Kaiser at bandwidth `4`,
normalized for exact `960/240` overlap reconstruction. Batch 29.6CW tests that
window on Signal's standard grid.

## Closed Lanes

- combined grid/window experiments and predictor-law changes
- parameter sweeps, corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CW source Kaiser-window attribution.
