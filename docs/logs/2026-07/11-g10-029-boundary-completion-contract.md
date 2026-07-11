# g10.029 Boundary Completion Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Batch 29.6P tests one bank:

- raw channels `0..1534` unchanged
- no global per-bin tightening
- channel `1535` replaced by one zero-delay real Nyquist completion
- completion width fixed to `16` channel spacings
- magnitude `sin(pi*smoothstep(s)/2)` with cubic smoothstep
- unchanged `1536` channels, `384`-frame hop, and complete canonical dual

No alternative width, taper, normalization, delay, or channel allocation may
be swept.

## Stop Gates

1. Complete-frame reconstruction must pass Contract `082` coverage,
   conditioning, dual-residual, exact-length, error, finite-value, and repeat
   gates.
2. Channels `0`, `15`, `16`, `768`, `1534`, and `1535` must pass the bounded
   `1e-12` dual-atom guard.
3. Only then may the all-channel guard scan run.
4. Only complete transform and guard passage may reopen derivative and
   projected-heap reproof.

No coefficient assembly, inverse audio synthesis, corpus, stereo, dynamic
ratio, or product route opens here.

## Next Task

Implement reconstruction and stop on any coverage, conditioning, dual,
identity-error, finite-value, hash, or representative-guard failure.
