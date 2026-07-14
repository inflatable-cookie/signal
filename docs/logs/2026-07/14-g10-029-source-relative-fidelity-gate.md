# Source-Relative Fidelity Gate

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CS
Scope: Rule 31G fidelity-gate correction and first internal differential

## Decision

Use paired pinned-source parity as the topology-fidelity rejection gate. Keep
`-60 dB` as an absolute diagnostic.

Signal may be no more than `1 dB` worse than pinned Signalsmith Stretch
revision `57b93f4e` for each exact quantized isolated tone and the chord. Exact
length, finiteness, pitch, repeat, transient, silence, boundary, fallback, and
mechanism gates remain unchanged.

## Result

- pinned absolute diagnostic failures: `[4 tones, 1 chord]`
- Signal source-relative failures: `[3 tones, 1 chord]`
- report direction: `SignalTranslationDivergence`
- real-source rendering: closed

The existing output hashes and measurements repeat unchanged. This batch
changes gate interpretation and report direction, not synthesis output.

## Internal Differential

Pinned source fractional frequency lookup returns zero beyond either spectrum
edge. Signal's translated lookup clamps to the nearest edge bin.

At ratio `2`, geometry `N=960`, `H=240`, and long distance `4`, ten vertical
observations per frame cross the low-frequency boundary:

- one short-lower observation
- one short-upper observation
- four long-lower observations
- four long-upper observations

Signal substitutes edge-bin energy and phase where pinned source contributes
zero. Its ascending correction order can carry low-bin decisions into higher
bins. This makes boundary policy a plausible cross-tone mechanism, but the
static source comparison does not prove causality.

## Closed Lanes

- production predictor changes
- weights, windows, geometry, distances, floors, and parameter sweeps
- corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CT. Add one report-only zero-extension variant and measure it
against both the frozen clamped translation and pinned source. Stop if it does
not materially close the paired failures.
