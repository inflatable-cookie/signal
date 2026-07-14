# Frequency-Boundary Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CT
Scope: report-only fractional frequency-boundary ablation

## Decision

Reject zero-extension as the source-parity-gap owner. Keep the frozen clamped
translation unchanged.

The variant changes only fractional spectrum lookup outside valid bins. It
matches pinned source by returning zero-valued neighbouring bins instead of
clamping to DC or Nyquist. Geometry, window, scheduling, phase equations,
weights, distances, normalization, fallback, and synthesis remain identical.

## Evidence

| Control | Clamped OOB | Zero-extended OOB | Zero minus clamped | Zero minus pinned |
| ---: | ---: | ---: | ---: | ---: |
| `110 Hz` | `-26.552889 dB` | `-26.586095 dB` | `-0.033206 dB` | `+19.430119 dB` |
| `164.8138 Hz` | `-37.759764 dB` | `-37.754081 dB` | `+0.005683 dB` | `+8.046467 dB` |
| `220 Hz` | `-23.543720 dB` | `-23.543651 dB` | `+0.000068 dB` | `+21.142629 dB` |
| `329.6276 Hz` | `-51.497422 dB` | `-51.497295 dB` | `+0.000126 dB` | `-6.225185 dB` |
| chord | `-30.236975 dB` | `-30.305355 dB` | `-0.068380 dB` | `+9.710904 dB` |

Paired failure counts remain `[3 tones, 1 chord]`. The variant preserves exact
`32000`-frame output, finiteness, pitch tolerance, and repeated hashes. Ten
lookup observations differ per processed `2x` frame, but their final-output
effect is negligible.

Zero-extension hashes:

- `110 Hz`: `ad83e502f2857859`
- `164.8138 Hz`: `d3ada3aca57d9041`
- `220 Hz`: `a066a6c2165f6fd1`
- `329.6276 Hz`: `68dd4e32317a46fb`
- chord: `35ecdf5467753361`

## Consequence

Static source differences are insufficient to select the next edit. The next
evidence must align internal source and Signal states and locate the first
material divergence before another ablation.

## Closed Lanes

- production predictor changes and compounded experimental variants
- weights, windows, geometry, distances, floors, and parameter sweeps
- corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CU. Trace one aligned interior frame at the analysis,
preliminary-horizontal, and corrected-output boundaries. Select the next
mechanism from the first material divergence.
