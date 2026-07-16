# g10.029 Stereo Synthesis Closure Attribution

Date: 2026-07-16
Scope: Batch 29.7G report-only linked-stereo synthesis trace

## Result

Ideal target-length quadrature tones measure within `1.110223e-13 rad` on the
whole record. Applying the production interior crop creates an absolute
estimator floor of `0.000142` to `0.000489 rad`. Stage comparison therefore
uses the identically cropped ideal IPD instead of nominal `pi/2` alone.

Current/oracle interior support-frame errors are:

| Ratio | Current | Oracle |
| --- | ---: | ---: |
| `0.75x` | `0.001134454` | `0.001144503` |
| `1.5x` | `0.000894103` | `0.000604311` |
| `2.0x` | `0.010634150` | `0.010644140` |

Whole boundary support frames reach `3.132584` to `3.141566 rad`. Overlap
accumulation often reduces the frame error, especially at `2.0x`.
Normalization changes whole/interior measurements by less than `1e-9 rad`.

All current/oracle row audio hashes remain exactly those frozen by 29.7F.
Evidence hash: `7f8cee549977896d`.

## Decision

The first observable post-spectrum divergence is real support-frame synthesis,
not coefficient projection, edge constraint, or normalization. The cropped
absolute metric has a real floor, but ideal-relative calibration does not
remove the render residual. Batch 29.8 remains closed.

## Next Task

Run Batch 29.7H analytic-overlap feasibility. Feed the same corrected spectra
through complex positive-frequency accumulation while preserving current
output and every frozen control.
