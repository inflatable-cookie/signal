# g10.029 Reference Projection Residual Attribution

Date: 2026-07-16
Scope: Batch 29.7F report-only linked-stereo attribution

## Result

The implemented recurrence remains frozen. Peer/reference complex relation
error is at most `4.440892e-16 rad` immediately after projection and after
real-edge constraint at `0.75x`, `1.5x`, and `2.0x`.

Whole/interior quadrature IPD error is:

| Ratio | Current whole | Current interior | Oracle whole | Oracle interior |
| --- | ---: | ---: | ---: | ---: |
| `0.75x` | `0.008194522` | `0.001651872` | `0.020384232` | `0.000984719` |
| `1.5x` | `0.007622820` | `0.000628105` | `0.007752772` | `0.000432013` |
| `2.0x` | `0.016073680` | `0.000070926` | `0.008543144` | `0.000469647` |

Boundary removal reduces tone error but does not close the steady image
residual. Current whole/interior correlated mid/side changes are
`0.434087/0.397904`, `0.267458/0.264677`, and `0.173960/0.178868 dB`.
The constant `pi/2` oracle improves some measurements and regresses others.
Per-bin input-relation variability is not a sufficient owner.

Evidence hash: `87a057697db91edd`. Current/oracle row audio hashes:

- `0.75x`: `b90271ab51a052f9` / `521245e5d1459e7e`
- `1.5x`: `ae096858dd2afcf6` / `f364ffe28d512f73`
- `2.0x`: `3c2c378ff8e912a4` / `e06fb9db7d5ac45c`

## Decision

Coefficient projection and real-edge constraint are excluded. The first
unexcluded seam is inverse synthesis, overlap accumulation, normalization, or
finite-record measurement. Batch 29.8 remains closed.

## Next Task

Run Batch 29.7G synthesis-closure attribution. Calibrate ideal whole/interior
IPD measurement and trace the frozen current/oracle paths through inverse,
overlap, and normalization stages before changing topology.
