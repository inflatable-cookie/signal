# g10.029 Boundary Completion Rejection

Date: 2026-07-11
Status: rejected at reconstruction conditioning

## Result

Batch 29.6P implemented the single frozen boundary-completion candidate. Raw
channels `0..1534` remain untightened and channel `1535` uses the frozen
zero-delay smoothstep-sine Nyquist completion.

The candidate covers every positive bin and exact canonical-dual identity
passes, but the frame is too poorly conditioned:

- frame minimum: `0.7361080720727322`
- frame maximum: `2.1937926703636856`
- condition ratio: `2.9802589505456814` (required at most `1.25`)
- canonical-dual residual: `7.657380695031188e-11`
- reconstruction peak error: `2.9103830456733704e-11`
- reconstruction RMS error: `4.781879301149326e-13`
- reconstruction head error: `4.83726210129265e-12`
- reconstruction tail error: `0`
- non-finite values: `0`
- preserved-channel hash: `899c7f7b775c1378`
- Nyquist-completion hash: `463ca8b834c318d5`

Evidence and hashes repeat exactly.

## Stop Decision

The condition gate fails before the representative dual-atom guard. No guard,
phase reproof, coefficient assembly, audio synthesis, corpus, stereo, dynamic
ratio, cache, or product-routing work is authorized.

Pointwise tightening previously fixed conditioning but created the channel `0`
boundary tail. Another completion-width or taper guess is not justified. Batch
29.6Q must first freeze one smooth endpoint-compatible frame preconditioner or
normalizer that retains raw channel `0` compactness and the frozen channel
`1535` completion while restoring condition ratio at most `1.25`.

## Next Task

Freeze the Batch 29.6Q preconditioner contract. Do not implement candidates.
