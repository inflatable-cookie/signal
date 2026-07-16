# Linked-Stereo Quality Attribution

Date: 2026-07-16
Roadmap: `g10.029`, Batches 29.7C and 29.7D reassessment
Scope: report-only fixed-ratio stereo quality and failure ownership

## Decision

Fail the quality gate. Keep stereo export, listening, dynamic ratio, routing,
and promotion closed. Return to cross-channel recurrence research before
changing Rule 31H or implementation.

## Quality Evidence

| Ratio | Max IPD rad | Delay in/out | Correlated M/S dB | Correlated corr | Max attack | Crossfeed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `0.75x` | `0.882660845` | `11/11` | `11.672982` | `0.809788` | `2` | `0` |
| `1.5x` | `1.526285478` | `11/8` | `11.990144` | `0.847887` | `1` | `0` |
| `2.0x` | `3.073635178` | `11/23` | `12.728013` | `0.937320` | `1` | `0` |

- `0` phase offset: exact at every tone and ratio
- `pi` phase offset: maximum error below `7.32e-13 rad`
- `pi/2` phase offset: fails at every tone and ratio
- decorrelated mid/side change: `0.078544` to `0.134587 dB`
- decorrelated correlation change: `0.009041` to `0.015495`
- isolated/dense replica failures: `0`
- silent-peer peak: exact `0`
- mechanics hash remains `426af565378e9ce1`
- repeat: exact

Frozen quality hashes:

- row audio: `ddc816d477db135d`, `6842967ca6c7984b`, `9d38e21d580f84ed`
- aggregate audio: `0509599cb46b0cfc`
- row measurement: `e5230b1bc9a0ddfa`, `e531b0f39f37fe94`, `8e1a7fe4eead7031`
- aggregate measurement: `2d8f8471d88cf383`

## Attribution

The same inputs were rendered as two independent coherent mono paths. Failure
masks are identical to linked stereo: `13`, `15`, and `15`. At `0.75x` and
`2.0x`, all four failing-family measurements are exact matches. At `1.5x`,
delay and image are exact; maximum IPD is `1.526285478` linked and
`1.526050345` independent. Aggregate shared-mode selection is not the primary
cause. The channel-local recurrence cannot preserve arbitrary interchannel
phase relationships.

- independent audio hashes: `85f21cc0e098ab18`, `13a38904092ee517`,
  `a3b8aaa5e25593e6`
- attribution evidence hash: `d148ae6a7114ef6a`

## Next Task

Research cross-channel recurrence in source-studied engines and canonical
phase-vocoder literature. Compare shared phase-increment and explicit
complex-ratio preservation without implementing either. Promote one
license-safe topology into architecture and Rule 31H before new stereo code.
