# Roadmap Generation Index

Status: active
Updated: 2026-03-14

## Active generation

- `g06`
- opened on 2026-03-13 after `g05` closeout to deepen runtime recovery,
  profiling, plugin-format breadth, hardware/external-I/O depth, media-service
  depth, and shared soak evidence for downstream consumers such as Loophole
- `g07` is now seeded as the next planned generation for spatial or
  multichannel depth, Linux-native runtime breadth, control-surface or MIDI
  hardware substrate, and fuller sample-domain time-stretch work

## Generation log

| Generation | Started | Reason | Notes |
| --- | --- | --- | --- |
| `g01` | 2026-03-08 | Initial Signal docs and migration sequence | Seeded after Northstar bootstrap and Finch research migration |
| `g02` | 2026-03-11 | Continue beyond the runtime baseline with reusable DSP and analysis depth | Closed on 2026-03-11 after shared spectral/resampling, rhythm, tonal, loudness, descriptor, embedding, and acceptance-spine work landed |
| `g03` | 2026-03-12 | Continue beyond analysis depth with engine-oriented runtime substrate work | Closed on 2026-03-12 after routed mixer topology, metering, automation playback, warp/render, plugin-chain execution, offline render/freeze, and hardening depth landed |
| `g04` | 2026-03-12 | Continue beyond engine depth with reusable-runtime productization, multicore execution, and consumer-facing contract work | Closed on 2026-03-12 after contract freezing, scheduler depth, deferred work policy, portability, plugin breadth, and generation-closeout conformance/release proof landed |
| `g05` | 2026-03-12 | Continue beyond the first stable Signal boundary with broader backend breadth, host-edge stability, publication-grade packaging, and downstream release confidence | Closed on 2026-03-13 after backend-neutral plugin breadth, shared host-edge contracts, publication-grade packaging manifests, downstream automation, and the combined generation-closeout proof landed |
| `g06` | 2026-03-13 | Continue beyond reusable-boundary closeout with runtime recovery depth, instrumentation, feature breadth, and Loophole-facing runtime evidence | Active; opened with a 20-milestone runway covering recovery/resumability, profiling, VST3/AU, MIDI/event depth, hardware/external-I/O, media services, and shared acceptance/soak promotion |
| `g07` | 2026-03-13 | Seed the post-`g06` feature-expansion queue around routing or multichannel depth, Linux-native breadth, control-surface substrate, and fuller time-stretch capability | Planned; seeded with a 20-milestone runway covering multichannel or spatial execution, LV2 and Linux backends, external MIDI/control surfaces, sample-domain stretch, and integrated acceptance depth |

## Rollover policy

Create a new generation when:
- manually triggered by maintainers based on sequencing needs.
- typically after a major vision/architecture shift or when roadmap scale warrants a new boundary.

## Next task

Continue `g06.006` with Batch 6.2 and keep the generation on Signal-owned
runtime surfaces by instrumenting the newly frozen per-block timing and
pressure seam before broader scheduler, plugin, hardware, media, and
acceptance lanes widen while `g07` remains the next full feature-breadth queue.
