# g10.029 Nyquist Alias-Coupling Ablation

Date: 2026-07-11
Status: passed; orthogonal or multi-row completion research selected

## Result

The release-only proof compares the full exact-pointwise frame operator,
complete channel-`1535` removal, and off-diagonal-only removal across all `11`
alias residues.

| Operator | Minimum | Maximum | Condition | Limiting residues |
| --- | ---: | ---: | ---: | --- |
| full | `0.5008176703` | `1.4982679809` | `2.9916436058` | `0`, `0` |
| channel removed | `0.3956619532` | `1.0483817856` | `2.6496906694` | `0`, `10` |
| channel diagonalized | `0.9409450361` | `1.0483817856` | `1.1141796230` | `0`, `10` |

The frozen minimum mode changes by `-0.0099565921` under complete removal and
`+0.4912819465` under diagonalization. The maximum changes by `-0.9945285028`
and `-0.4922754605`. This matches the prior signed cross-term attribution.

Maximum Jacobi errors are residual `6.6651241979e-13`, orthogonality
`8.1986920111e-15`, trace `8.8817841970e-16`, and Frobenius
`1.1825331516e-14`. Maximum subtraction closure is `2.2230129165e-16`.
Filter hash `83802ce56ffc0e29` and evidence hash `eeef1e5788727c03` repeat exactly.

## Decision

Removing cross-bin coupling while retaining channel `1535` diagonal energy
passes the `1.25` condition gate. Removing the whole channel does not. Freeze
orthogonal or multi-row Nyquist-completion research; do not replace the useful
diagonal energy or broaden to all high-edge channels.

## Boundary

This is a matrix ablation, not a filter or synthesis candidate. Duals, guards,
phase, synthesis, corpus rendering, stereo, dynamic ratio, and product routing
remain closed.

## Next Task

Freeze Batch 29.6Z orthogonal or multi-row Nyquist-completion research.
