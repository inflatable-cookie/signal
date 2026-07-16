# Reference-Relative Recurrence Proof

Date: 2026-07-16
Roadmap: `g10.029`, Batch 29.7E
Scope: report-only fixed-ratio linked-stereo recurrence and unchanged quality gate

## Decision

Retain the reference-relative implementation as evidence, but fail the quality
gate. It fixes delay and removes most image/IPD damage. The exact IPD ceiling
and two correlated-image rows remain open. Do not export listening audio or
tune thresholds.

## Mechanics

- mono, duplicate, hard pan, swap, polarity, gain, silence, coverage,
  boundaries, crossfeed, and repeat pass
- both channels own non-zero bins
- exact ties choose channel zero
- owner switches: `512`, `512`, `860`
- switch-boundary step growth: `-1.080220`, `-0.583075`, `-0.413230 dB`
- mechanics audio hash: `28803a9f2e5bd83e`
- mechanics evidence hash: `03b66c25196493c2`

## Quality

| Ratio | Worst IPD rad | Delay in/out | Correlated M/S dB | Correlation delta |
| --- | ---: | ---: | ---: | ---: |
| `0.75x` | `0.008194522` | `11/11` | `0.434086623` | `0.012992762` |
| `1.5x` | `0.007622820` | `11/11` | `0.267458408` | `0.007923982` |
| `2.0x` | `0.016073680` | `11/11` | `0.173959862` | `0.005427491` |

Decorrelated image, transient timing, replicas, silent-peer crossfeed, and
repeat pass. Quality audio hash is `a5fb675cb0484eda`; measurement hash is
`ae77c422ea75e292`; residual attribution hash is `ebfb64802f96d50b`.

## Attribution

Independent recurrence failure masks remain `[13, 15, 15]`. Reference-relative
recurrence reduces them to `[5, 5, 1]`. The remaining fault is downstream of
the primary recurrence correction or inside the instantaneous per-bin relation
field. Batch 29.7F must locate that boundary before another topology change.

## Next Task

Measure relation error after projection and real-edge constraint, compare
interior and whole output, and run a known-constant-relation oracle. Keep Batch
29.8 closed.
