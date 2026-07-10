# g10.029 Common-Grid Phase Alias Rejection

Date: 2026-07-10
Status: rejected

## Evidence

The tone diagnostic estimates heterodyned horizontal phase motion and removes
each deterministic channel delay before adjacent-channel comparison.

- `312.5 Hz`: frequency error `2.375735e-8`, residual `4.910683e-6`
- `1 kHz`: frequency error `1.478986e-7`, residual `1.263033e-5`
- `8 kHz`: frequency error `0.065450362`, residual `0.243248864`

The low and middle controls prove the compensation sign. The high control
exposes phase aliasing: hop `384` resolves at most `+/-62.5 Hz` of heterodyned
motion, less than the high-frequency wavelet bandwidth.

## Decision

Reject inter-column phase differences as the common-grid
instantaneous-frequency estimator. Do not implement fractional projection,
heap integration, synthesis, or corpus rendering on top of it. Keep the passing
reconstruction transform.

## Next Task

Research auxiliary derivative-filter or reassignment estimators that do not
depend on wrapped phase motion across the `384`-frame hop.
