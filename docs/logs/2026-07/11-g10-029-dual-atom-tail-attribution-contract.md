# g10.029 Dual-Atom Tail Attribution Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Batch 29.6O attributes the rejected common-grid tail without changing the
transform. Measure channels `0`, `15`, `16`, `768`, and `1535` through raw
analysis, tightened analysis, and exact tightened canonical-dual stages. Keep
positive-only analytic and conjugate-mirrored real-output atoms separate.

## Fixed Matrix

- radii: `384`, `1536`, `4096`, `8192`, `12288`, `16000` frames
- thresholds: `1e-6`, `1e-8`, `1e-10`, `1e-12`
- attribution: tightening, dualization, mirroring, channel `0/16`, channel
  `0/768`
- probe transform: `34176` frames

Report complete finite evidence, exact dual residual at most `1e-8`, stable
stage/channel hashes, and exact repeat evidence. Declared infinite attribution
ratios are allowed only for true zero denominators.

## Boundary

Do not tune filters, geometry, thresholds, or solver policy. Do not assemble
coefficients, synthesize audio, render the corpus, open stereo/dynamic work, or
change product routing.

## Next Task

Implement Batch 29.6O and stop at the evidence-led redesign checkpoint.
