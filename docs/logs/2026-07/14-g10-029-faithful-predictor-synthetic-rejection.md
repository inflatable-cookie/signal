# Faithful Predictor Synthetic Rejection

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CM
Scope: report-only mono synthetic proof

## Decision

Reject the complete faithful-predictor proof before real-source rendering.

The implementation passes every frozen hard check except chord/pad out-of-band
energy. The four-tone control measures `-30.200611 dB` against `-60 dB`. The
unprocessed control measures about `-80.43 dB` with the same Hann FFT analysis,
so the failure is predictor-created modulation rather than analysis leakage.

Do not tune intervals, windows, distances, weights, floors, or update order.
The next batch is trace-only sideband attribution inside the frozen topology.

## Implemented Topology

- sample-rate-derived `H = round(0.03 * sample_rate)`, `N = 4H`
- fixed output grid and rounded inverse-ratio input centres
- auxiliary horizontal observation at current input centre minus `H`
- actual rounded input hop used only for the local vertical time factor
- fractional short/long lower and upper frequency twists
- ascending correction with corrected lower and preliminary upper state
- target-energy normalization and energy-relative current-input fallback
- real DC/Nyquist, centered reflection, exact overlap normalization and crop
- identity bypass

The implementation lives only in the release-test source-study lane. Production
routing and cache identity are unchanged.

## Evidence

At `8 kHz`, geometry is `H=240`, `N=960`, overlap four. This preserves the
frozen 120/30 ms shape and the same normalized bin geometry as higher sample
rates.

| Gate | Result |
| --- | ---: |
| structural failures: length/finite/coverage/boundary/hash | `[0,0,0,0,0]` |
| maximum bass error | `0.007404 Hz` |
| octave failures | `0` |
| maximum chord peak error | `0.019739 Hz` |
| input chord out-of-band energy | `-80.429254 dB` |
| chord out-of-band energy | `-30.200611 dB` |
| maximum event error | `255 frames` |
| midpoint replica failures | `0` |
| silence peak | `0` |
| horizontal predictions | `338624` |
| short lower / upper | `337920 / 337920` |
| long lower / upper | `335808 / 335808` |
| corrected / fallback | `205265 / 133359` |
| repeat hash | `a66c6564847ede88` |

The event result passes by one frame and remains a visible risk, but it does not
own the stop. The sideband failure is 29.80 dB above the frozen ceiling.

## Source-Audit Correction

The preceding contract said the actual input hop drives horizontal transport.
Pinned-source reinspection corrected that statement before implementation. The
horizontal complex ratio uses an auxiliary input spectrum one fixed output
interval behind the current projected input centre. The actual rounded input
hop supplies the vertical time factor. Memo 005, contract 082, architecture,
dossier, and roadmap now agree.

## Closed Lanes

- real-source corpus and holdout
- listening export
- stereo and dynamic ratio
- production routing and cache
- parameter and factor sweeps

## Next Task

Define Batch 29.6CN as trace-only attribution. Measure the four-tone sideband
energy after preliminary horizontal transport, each vertical direction and
distance, ascending correction, normalization/fallback, and overlap synthesis.
Do not change the frozen topology until one stage owns the failure.
