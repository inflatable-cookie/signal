# Analysis Representation Interaction

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CX
Scope: pinned, report-only analysis-representation `2x2`

## Decision

Retain the exact combined analysis representation for further research. Do not
promote either component alone.

The experiment holds `960/240` support, scheduling, predictor equations,
distances, normalization, fallback, boundary policy, and synthesis ownership
fixed. It compares Signal baseline, modified-grid-only, periodic-Kaiser-only,
the exact combined cell, and pinned Signalsmith Stretch on the same quantized
controls.

## Evidence

| Control | Baseline | Grid only | Window only | Combined | Pinned | Interaction | Combined minus pinned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `110 Hz` | `-26.552889` | `-32.623549` | `-36.630974` | `-46.157030` | `-46.016214` | `-3.455397` | `-0.140817` |
| `164.8138 Hz` | `-37.759764` | `-33.432946` | `-31.853466` | `-45.653754` | `-45.800547` | `-18.127106` | `+0.146793` |
| `220 Hz` | `-23.543720` | `-20.373037` | `-32.366383` | `-44.563946` | `-44.686281` | `-15.368246` | `+0.122334` |
| `329.6276 Hz` | `-51.497422` | `-22.504905` | `-20.733038` | `-45.143285` | `-45.272110` | `-53.402764` | `+0.128825` |
| chord | `-30.236975` | `-26.500523` | `-28.415590` | `-40.657139` | `-40.016259` | `-15.978002` | `-0.640880` |

All values are out-of-band energy in dB except the explicitly labelled
interaction and delta columns. The interaction is `combined - grid - window +
baseline` on the frozen dB diagnostic.

- baseline paired failures: `[3 tones, 1 chord]`
- grid-only failures: `[4 tones, 1 chord]`
- window-only failures: `[4 tones, 1 chord]`
- combined failures: `[0 tones, 0 chord]`
- identity maximum error: `2.220446e-16`
- length, coverage, finiteness, boundaries, pitch, and repeat failures: zero

Combined output hashes:

- `110 Hz`: `1497ff00420ebf4e`
- `164.8138 Hz`: `34d3f1e18ab56752`
- `220 Hz`: `1dda3a2c0163ac8f`
- `329.6276 Hz`: `11465d184b111c89`
- chord: `d23cd768f2a461bd`

## Consequence

Spectral prediction is coupled to the full analysis phase basis. The source
window and transform grid are not independently useful settings; together they
reconstruct the observed source fidelity. Batch 29.6CY carries this coherent
representation through the complete synthetic proof before any real-source
confirmation.

## Closed Lanes

- predictor-law changes, third mechanisms, and parameter sweeps
- real-source corpus, listening, stereo, and dynamic ratio
- production dependency, cache, routing, and promotion

## Next Task

Run Batch 29.6CY coherent-representation synthetic gate.
