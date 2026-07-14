# Faithful Predictor Sideband Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CN
Scope: trace-only mono synthetic attribution

## Decision

Preliminary horizontal transport owns the earliest four-tone sideband failure.
The defect is frame-rate modulation, not a stationary pitch offset.

Do not change vertical distances, normalization, fallback, overlap synthesis,
or production routing. Batch 29.6CO compares isolated and mixed horizontal
observations before choosing an observation-geometry or equation correction.

## Method

Six trace views synthesize observations from the same frozen complete predictor
state:

- preliminary horizontal
- short lower
- short upper
- long lower
- long upper
- complete correction

The individual views change observation only. They do not replace carried
predictor state. One separate oracle analyses and synthesizes the exact steady
chord on the output grid through the same square-root-Hann overlap operator.

Normalization phase delta and significant current-input fallback are recorded
from the complete chord render. No real source is read.

## Evidence

| Stage | Out of band | Strongest spur | Nearest-tone offset | Frame-grid error | Hash |
| --- | ---: | ---: | ---: | ---: | --- |
| horizontal | `-28.182097 dB` | `76.660156 Hz` | `33.339844 Hz` | `0.006510 Hz` | `6ae9748d411e5b8f` |
| short lower | `-31.952348 dB` | `192.382812 Hz` | `27.569013 Hz` | `5.764321 Hz` | `fe6142e03ec86aa7` |
| short upper | `-30.370023 dB` | `198.242188 Hz` | `21.757812 Hz` | `11.575521 Hz` | `5a979681d3a39606` |
| long lower | `-28.862436 dB` | `76.660156 Hz` | `33.339844 Hz` | `0.006510 Hz` | `dbc8ee794f0d2278` |
| long upper | `-18.107883 dB` | `317.871094 Hz` | `11.756506 Hz` | `11.756506 Hz` | `00da6b2936497fb0` |
| complete | `-30.200611 dB` | `76.660156 Hz` | `33.339844 Hz` | `0.006510 Hz` | `61b560b628b11e9d` |

The output frame rate is `33.333333 Hz`. Horizontal and complete output share a
spur one frame rate below the `110 Hz` component. All main peaks remain within
the prior `0.5 Hz` gate, excluding a fixed pitch shift as the dominant defect.

Closure checks:

- exact analysis/synthesis overlap oracle: `-80.392196 dB`
- maximum normalization phase delta: `4.441e-16` radians
- significant fallback count: `0`
- repeated stage evidence and hashes: exact

Overlap synthesis can reproduce the same steady chord below the `-60 dB` gate.
Normalization is a positive real scale and introduces no meaningful phase
change. Fallback does not touch significant chord bins. Vertical predictions
are all already above the sideband ceiling and cannot be the earliest owner;
complete correction reduces horizontal leakage by only `2.018168 dB` while
retaining its frame-rate spur.

## Closed Lanes

- predictor and observation changes
- parameter sweeps
- corpus, holdout, and listening
- stereo and dynamic ratio
- cache and production routing

## Next Task

Run Batch 29.6CO. Render the four chord tones separately and together through
the unchanged horizontal trace. Measure nearest-bin auxiliary-ratio phase-
advance variance and frame-rate sidebands. Choose observation-geometry redesign
only if mixing creates the defect; otherwise inspect the horizontal phase
equation and synthesis attachment.
