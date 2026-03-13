# 2026-03-13 22:38:43 GMT - Chorus feature sweep and post-g06 candidate suite

## Summary

Swept Chorus for reusable runtime-facing feature demand that is not yet clearly
captured by active Signal `g06` milestones, then parked the resulting
post-`g06` suite in Signal backlog instead of diluting the current generation.

## Key findings

The strongest Chorus-derived Signal candidates not yet explicit in `g06` are:

- spatial and multichannel execution depth
- sidechain, auxiliary, and multi-bus runtime behavior
- complex plugin I/O such as multi-output instruments and sidechain-capable FX
- Linux-native plugin breadth beyond CLAP/VST3, especially LV2
- Linux hardware backend depth across ALSA, JACK, and PipeWire
- external MIDI, control-surface, and feedback-device substrate
- fuller sample-domain time-stretch beyond the current bounded warp contract

## Changes

- added backlog item:
  - `docs/roadmaps/backlog/post-g06-chorus-feature-expansion-and-g07-candidate-suite.md`
- updated:
  - `docs/roadmaps/generation-index.md`
  - `docs/roadmaps/README.md`

## Why backlog instead of active queue

`g06` already has a coherent 20-milestone runway around recovery, profiling,
plugin baseline breadth, hardware supervision, media services, and soak
evidence. The Chorus-derived feature sweep found a second coherent suite, but
it is feature-expansion work rather than a reason to destabilize the current
active queue.

Parking the suite in backlog now keeps two things true:

- Signal has a durable next-generation target with concrete milestones
- the current execution thread can stay focused on `g06.001` onward

## Validation

- `effigy qa:docs`
- `git diff --check`

## Next Task

Continue `g06.001`, and when maintainers want the next generation opened,
promote the new post-`g06` candidate suite by freezing multichannel and routing
substrate first so later spatial, sidechain, Linux, control-surface, and
time-stretch work all share one base.
