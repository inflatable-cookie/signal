# g10.029 Auxiliary Derivative Estimator Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Replace aliased inter-column phase differences with a same-column
time-derivative filter ratio. Derive each auxiliary response from the final
tightened Batch 29.6J analysis filter. Estimate absolute instantaneous
frequency from the derivative/original cross-ratio, then apply the already
proven channel-delay compensation.

## Stop Gate

Periodic `312.5 Hz`, `1 kHz`, `8 kHz`, and `19.5 kHz` controls must meet the
`1e-6` radians/sample frequency limit and `2e-5` compensated phase limit.
Silence must skip zero-energy ratios. Noise, all reported values, and hashes
must remain finite and deterministic.

No fractional projection, phase heap, synthesis, corpus render, stereo, or
product route opens in this proof.

## Next Task

Implement Batch 29.6L and stop on sign, scale, high-band, finite-value, delay,
or determinism failure.
