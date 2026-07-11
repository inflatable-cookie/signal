# g10.029 Boundary Preconditioner Contract

Date: 2026-07-11
Status: decision frozen

## Problem

The untightened boundary bank reconstructs exactly but has condition ratio
`2.9802589505`. The earlier pointwise tightener restored conditioning while
raising channel `0` real-output tail energy from `1.622121e-13` to
`6.270779e-7`. Its positive-frequency scale is not smooth under endpoint
mirroring.

## Decision

Batch 29.6R tests one common real scalar normalizer. For raw-bank energy
`E(f)`, use `r(f)=1/sqrt(E(f))` in the interior. Across the existing `16h` DC
and Nyquist spans, blend `r(f)` to its exact endpoint value with quintic
smootherstep `b(s)=6s^5-15s^4+10s^3`.

The multiplier is common to all channels. Raw channel supports, phases,
delays, geometry, and the frozen Nyquist completion do not change. The scalar
has zero first and second one-sided derivatives at DC and Nyquist. Hash the raw
bank and multiplier separately.

Do not sweep width or taper, fit endpoint slopes, add per-channel gains, run a
second correction pass, or change the completion.

This rule is a Signal inference from invertible nonstationary-Gabor frame
construction, not a published endpoint-normalization recipe. Frame bounds and
canonical-dual reconstruction follow the verified construction boundary in
[Holighaus et al.](https://arxiv.org/abs/1210.0084) and
[Dörfler and Matusiak](https://arxiv.org/abs/1112.5262). Measured atom guards
remain authoritative for time localization.

## Stop Gates

1. Reconstruction must pass condition ratio `1.25`, dual residual `1e-8`,
   existing identity errors, coverage, finite values, hashes, and repeat gates.
2. Only then run the frozen six-channel representative guard at excluded energy
   `1e-12` and maximum support radius `16384` frames.
3. Stop before the all-channel guard on either failure.
4. Keep phase reproof and every synthesis surface closed.

## Next Task

Implement Batch 29.6R through reconstruction. Run representative guards only
after reconstruction passes.
