# g10.029 Auxiliary Derivative Estimator Proof

Date: 2026-07-11
Status: passed

## Result

Batch 29.6L passes. Each finalized tightened analysis filter now has a
same-grid time-derivative auxiliary response. The imaginary
derivative/original cross-ratio estimates absolute instantaneous angular
frequency without inter-column phase unwrap.

Each column uses its maximum-energy qualified channel as one deterministic
coherent carrier estimate. This avoids leakage bias from weaker overlapping
filters. The same carrier compensates the strongest qualified adjacent-channel
pair for the known digital-delay difference.

## Evidence

| Tone | Max frequency error (rad/sample) | Max compensated residual (rad) |
| --- | ---: | ---: |
| 312.5 Hz | `6.938894e-18` | `1.776357e-15` |
| 1 kHz | `8.326673e-17` | `1.065814e-14` |
| 8 kHz | `6.217249e-15` | `1.261213e-12` |
| 19.5 kHz | `3.614442e-12` | `8.683081e-10` |

All values clear the frozen `1e-6` frequency and `2e-5` residual limits.
Silence produces no qualified ratios and records zero-energy skips.
Deterministic noise remains finite. Repeated reports, auxiliary hashes, and
trace hashes match exactly.

## Boundary

No fractional projection, heap integration, synthesis, corpus render, stereo,
or product route opens in this batch.

## Next Task

Freeze the fractional source-projection and bounded deterministic heap proof.
