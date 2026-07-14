# Stage-Aligned Source Trace

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6CU
Scope: pinned, report-only internal-state trace

## Decision

Assign the first pinned-source versus Signal divergence to the analysis
transform grid. Do not change another predictor equation.

Both paths use `960`-frame support, a `240`-frame interval, ratio `2`, and exact
quantized `8 kHz` controls. At aligned source centre `8400`, pinned Signalsmith
Stretch revision `57b93f4e` uses Signalsmith Linear revision `56686735`
(`0.3.1`). Linear chooses a `1024`-point modified real transform with `512`
half-bin bands starting at `3.90625 Hz`, spaced `7.8125 Hz`. Signal uses a
`960`-point standard real transform with `481` bins starting at DC, spaced
`8.333333 Hz`.

## Evidence

| Control | Stage | Pinned hash | Signal hash | Normalized magnitude delta |
| --- | --- | --- | --- | ---: |
| `110 Hz` | current | `900c6f814d64d4e5` | `bcd803b5f1369855` | `0.145164` |
| `110 Hz` | preliminary | `7aca7cb52a254a16` | `5faf1f540aebe235` | `0.144846` |
| `110 Hz` | corrected | `0bf939cef8ec3304` | `a205cce33610e027` | `0.145164` |
| `220 Hz` | current | `a2203b8042566f4d` | `0d682dcda081bea2` | `0.022231` |
| `220 Hz` | preliminary | `6676ee836d754e9d` | `29ce3d32ac17f9ac` | `0.022150` |
| `220 Hz` | corrected | `a67481f452c6f55e` | `5faf3ed86ad895a4` | `0.022231` |
| chord | current | `d242be289a89286c` | `7411d6fefdd850ab` | `0.081406` |
| chord | preliminary | `2b65e1d09edac920` | `b4b31faec3448c14` | `0.085683` |
| chord | corrected | `98d5b0da848b32af` | `1b50e5e21f58d59a` | `0.081406` |

Maximum target-bin relative phase deltas are `1.700284`/`1.700234 rad` for
`110 Hz`, `2.242243`/`2.242500 rad` for `220 Hz`, and
`2.806515`/`2.815586 rad` for the chord at preliminary/corrected boundaries.
All hashes and metrics repeat across two independent fixture runs.

The pinned preliminary state is reconstructed from the exact observed current,
prior input, prior output, and energy states using the pinned horizontal law.
The corrected state is read after upstream vertical prediction. Signal captures
the corresponding three states directly.

The magnitude and phase values do not prove a downstream predictor defect.
Bins from different transform sizes and origins are different basis functions.
That non-isomorphism occurs at the current-input spectrum, before horizontal
or vertical prediction, and is the earliest material divergence.

## Consequence

Batch 29.6CV tests one report-only transform-grid variant: retain the
`960`-frame support and Signal window, but use the pinned `1024`-point modified
half-bin representation. Hold scheduling, equations, normalization, fallback,
boundary policy, and synthesis ownership fixed. Read parity only after
identity, duration, finiteness, pitch, and repeat gates pass.

## Closed Lanes

- predictor-law and window changes
- combined transform/window experiments and parameter sweeps
- corpus, holdout, listening, stereo, and dynamic ratio
- external production dependency, cache, and routing

## Next Task

Run Batch 29.6CV modified analysis-grid attribution. Stop on rejection.
