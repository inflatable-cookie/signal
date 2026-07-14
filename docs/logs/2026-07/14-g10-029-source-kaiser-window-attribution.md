# Source Kaiser Window Attribution

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CW
Scope: pinned, report-only analysis/synthesis window ablation

## Decision

Reject the pinned Kaiser window as a standalone source-parity mechanism.
Correct its classification from symmetric to periodic.

The fixture reads coefficients directly from pinned Signalsmith Stretch
revision `57b93f4e` and Signalsmith Linear revision `56686735`. Stretch's
explicit Kaiser selection overwrites Linear's initial confined-Gaussian
configuration. The variant changes only Signal's analysis and synthesis
window. Its standard `960`-point grid, schedule, predictor equations,
distances, normalization, fallback, boundary policy, and overlap ownership
remain fixed.

## Window Provenance

- analysis hash: `cd811c4f82d161be`
- synthesis hash: `cd811c4f82d161be`
- analysis/synthesis maximum delta: `0`
- maximum endpoint-mirror delta: `0.002531886`
- four-hop overlap-product hash: `6dadf0c986c4bd49`
- maximum overlap unity error: `8.953040e-8`

The non-zero mirror delta is expected for Linear's even-length periodic Kaiser
sampling. The coefficients still form the exact source analysis/synthesis
pair.

## Structural Evidence

- analysis/synthesis identity maximum error: `2.775558e-16`
- output length, coverage, finiteness, and boundary failures: zero
- maximum tone peak error: `0.007328 Hz`
- chord peak error: `0.007460 Hz`
- all output hashes repeat

## Fidelity Evidence

| Control | Pinned | Signal baseline | Kaiser window | Window minus baseline | Window minus pinned |
| --- | ---: | ---: | ---: | ---: | ---: |
| `110 Hz` | `-46.016214 dB` | `-26.552889 dB` | `-36.630974 dB` | `-10.078085 dB` | `+9.385240 dB` |
| `164.8138 Hz` | `-45.800547 dB` | `-37.759764 dB` | `-31.853466 dB` | `+5.906298 dB` | `+13.947081 dB` |
| `220 Hz` | `-44.686281 dB` | `-23.543720 dB` | `-32.366383 dB` | `-8.822663 dB` | `+12.319897 dB` |
| `329.6276 Hz` | `-45.272110 dB` | `-51.497422 dB` | `-20.733038 dB` | `+30.764384 dB` | `+24.539072 dB` |
| chord | `-40.016259 dB` | `-30.236975 dB` | `-28.415590 dB` | `+1.821386 dB` | `+11.600669 dB` |

Hashes:

- `110 Hz`: `99aee1fb82b4aea1`
- `164.8138 Hz`: `7a22fba53bb2e333`
- `220 Hz`: `cfd6c08b12f29e5f`
- `329.6276 Hz`: `22619018da9f7247`
- chord: `943f51d03c8bb374`

Source-relative failures worsen from `[3 tones, 1 chord]` to
`[4 tones, 1 chord]`. The window does not produce coherent parity movement.

## Consequence

Do not promote the window alone. Grid-only and window-only main effects are now
both measured, while the pinned engine uses both together. Batch 29.6CX
completes this bounded `2x2` interaction without adding a third mechanism.

## Closed Lanes

- predictor-law changes and third analysis mechanisms
- parameter sweeps, corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CX pinned analysis-representation interaction.
