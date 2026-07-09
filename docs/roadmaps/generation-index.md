# Roadmap Generation Index

Status: active
Updated: 2026-07-09

## Current generation posture

- `g10` is the active generation. Its front door is
  `docs/roadmaps/g10/README.md`. Phase one (001-009) completed the
  deep-audit remediation program. Phase two (010-020) completed the engine
  build-out on the surviving seed. Phase three (021-025) established
  first-party stretch evidence, OfflineHighQuality DSP depth, offline artifact
  scale, RealtimePreview, and a deferred product-workflow checkpoint. The
  active structural stretch lane is `g10.026`, callback-safe RealtimePreview
  state.
- audit evidence: `chorus/research/2026-06-11-signal-deep-audit.md` (phase
  one) and `docs/research/2026-06-11-post-demolition-assessment.md` (phase
  two)
- rebuild queue: `docs/roadmaps/backlog/post-g10-rebuild-on-demand.md`
- the earlier post-`g08` backlog item remains informative context in
  `docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`

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
| `g08` | 2026-03-19 | Continue beyond bounded feature-expansion closure with live Linux backend ownership, richer plugin or device protocol depth, immersive routing, and workflow-adjacent runtime services | Closed on 2026-03-22 after live Linux ownership, LV2 or plugin protocol depth, immersive render breadth, device-protocol substrate, preview workflows, grouped acceptance lanes, integrated acceptance, and final generation-closeout work landed |
| `g10` | 2026-06-11 | Turn the 2026-06-11 deep audit into a remediation program: fix the real audio path, demolish simulated/narration mass (~70-80k LoC), consolidate hygiene, defer rebuilds to demand | Active; `g10.026` is the structural stretch lane |
| `g09` | 2026-04-08 | Turn the audit findings into a contract-backed realization, hardening, and interactive-proof program | Closed on 2026-04-11 after plugin and backend realization, production-readiness gating, and operator-visible demo proof landed |

## Rollover policy

Create a new generation only when maintainers explicitly decide the sequencing baseline needs a real reset.

Generations should be substantial. As a healthy default, expect something closer to 20 to 40 roadmap files before rollover is worth discussing. Treat that as a judgment guardrail, not an automatic counter.

Rollover is a closeout event, not a convenience move. Before opening the next generation:

- close, pause, supersede, or rehome every roadmap in the current generation
- refresh the roadmap front doors so the old generation is visibly closed
- purge stale generation-specific specs from `docs/specs/` so the active planning tree no longer carries dead lane debris

If that cleanup has not happened, stay in the current generation and finish the closeout there first.

## Next Task

Use `docs/roadmaps/g10/README.md` as the current active-generation front door.
Start `g10.026` Batch 26.2 only when implementing real streaming DSP state.
