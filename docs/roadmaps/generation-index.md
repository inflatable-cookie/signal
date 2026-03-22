# Roadmap Generation Index

Status: active
Updated: 2026-03-22

## Active generation

- `g08`
- promoted into the active queue on 2026-03-19 after `g07` closeout to deepen
  live Linux audio backend ownership, richer plugin or device protocol depth,
  immersive routing breadth, and runtime-owned preview or workflow services
- `g07` is now complete and remains the immediate bounded feature-expansion
  predecessor to the active live-ownership queue

## Generation log

| Generation | Started | Reason | Notes |
| --- | --- | --- | --- |
| `g01` | 2026-03-08 | Initial Signal docs and migration sequence | Seeded after Northstar bootstrap and Finch research migration |
| `g02` | 2026-03-11 | Continue beyond the runtime baseline with reusable DSP and analysis depth | Closed on 2026-03-11 after shared spectral/resampling, rhythm, tonal, loudness, descriptor, embedding, and acceptance-spine work landed |
| `g03` | 2026-03-12 | Continue beyond analysis depth with engine-oriented runtime substrate work | Closed on 2026-03-12 after routed mixer topology, metering, automation playback, warp/render, plugin-chain execution, offline render/freeze, and hardening depth landed |
| `g04` | 2026-03-12 | Continue beyond engine depth with reusable-runtime productization, multicore execution, and consumer-facing contract work | Closed on 2026-03-12 after contract freezing, scheduler depth, deferred work policy, portability, plugin breadth, and generation-closeout conformance/release proof landed |
| `g05` | 2026-03-12 | Continue beyond the first stable Signal boundary with broader backend breadth, host-edge stability, publication-grade packaging, and downstream release confidence | Closed on 2026-03-13 after backend-neutral plugin breadth, shared host-edge contracts, publication-grade packaging manifests, downstream automation, and the combined generation-closeout proof landed |
| `g06` | 2026-03-13 | Continue beyond reusable-boundary closeout with runtime recovery depth, instrumentation, feature breadth, and Loophole-facing runtime evidence | Closed on 2026-03-16 after recovery/resumability, profiling, VST3/AU, MIDI/event depth, hardware/external-I/O, media services, integrated acceptance, bounded soak, and generation-closeout promotion work landed |
| `g07` | 2026-03-13 | Seed the post-`g06` feature-expansion queue around routing or multichannel depth, Linux-native breadth, control-surface substrate, and fuller time-stretch capability | Closed on 2026-03-19 after multichannel or spatial execution, LV2 and Linux backends, external MIDI/control surfaces, sample-domain stretch, integrated acceptance, and generation-closeout promotion work landed |
| `g08` | 2026-03-19 | Continue beyond bounded feature-expansion closure with live Linux backend ownership, richer plugin or device protocol depth, immersive routing, and workflow-adjacent runtime services | Active; promoted on 2026-03-19 with a 20-milestone runway covering Linux live ownership, LV2 or plugin protocol depth, immersive render breadth, device-protocol substrate, preview workflows, and integrated acceptance. `g08.015` is active, and Batch 15.1 is now frozen as the shared cross-backend device workflow acceptance contract |

## Rollover policy

Create a new generation when:
- manually triggered by maintainers based on sequencing needs.
- typically after a major vision/architecture shift or when roadmap scale warrants a new boundary.

## Next task

Continue `g08.015` with Batch 15.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared cross-backend device protocol and live
workflow seam while keeping backend-specific depth explicit and non-blocking.
