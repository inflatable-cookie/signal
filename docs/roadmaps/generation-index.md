# Roadmap Generation Index

Status: active
Updated: 2026-03-12

## Active generation

- `g03`

## Generation log

| Generation | Started | Reason | Notes |
| --- | --- | --- | --- |
| `g01` | 2026-03-08 | Initial Signal docs and migration sequence | Seeded after Northstar bootstrap and Finch research migration |
| `g02` | 2026-03-11 | Continue beyond the runtime baseline with reusable DSP and analysis depth | Closed on 2026-03-11 after shared spectral/resampling, rhythm, tonal, loudness, descriptor, embedding, and acceptance-spine work landed |
| `g03` | 2026-03-12 | Continue beyond analysis depth with engine-oriented runtime substrate work | Opened for routed mixer topology, metering, automation playback, warp/render, plugin-chain execution, offline render/freeze, and hardening depth |

## Rollover policy

Create a new generation when:
- manually triggered by maintainers based on sequencing needs.
- typically after a major vision/architecture shift or when roadmap scale warrants a new boundary.

## Next task

Continue `g03.007` with artifact/parity hardening now that the runtime offline
render engine path has landed behind the request and recall-handoff contract.
