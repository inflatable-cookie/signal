# 2026-03-11 — g01.006 routed tail and latency propagation

## Summary

Finished the remaining routing-core tranche for `g01.006` by making latency and
tail contribution explicit in graph-owned execution structures and runtime
reporting.

## What Changed

- added per-node `tail_samples` to `signal-graph` node specs so routed topology
  metadata can declare decay contribution alongside latency
- propagated latency and tail through graph-owned bus preparation and execution
  state instead of treating them as host-local side knowledge
- extended `GraphRoutingSummary` and `GraphBlockReport` with routed tail metrics
  including total node tail, max node tail, output tail, and max routed-bus
  tail
- updated the runtime seam so `RuntimeEngineBlockSnapshot` captures the new tail
  metrics and exposes them through compact, multiline, and JSON observation
  rendering
- added routed graph fixtures covering direct-chain tail propagation, fan-in
  longest-path tail selection, and send/return wet-path tail accumulation

## Why It Matters

`signal-graph` now owns both routing shape and routed time contribution in a
way runtime can consume directly. That closes the main `006.2` gap and gives
the upcoming parameter-event work a graph seam that already knows how long
effects persist after signal flow moves through latency and tail-bearing nodes.

## Validation

- `cargo test -p signal-graph`
- `cargo test -p signal-runtime --no-run`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Deferred / Next

Begin `006.3` by defining graph-owned parameter-event batches and bounded
sub-block application rules, then prove them with deterministic gain, filter,
and delay node fixtures on top of the routed execution baseline.
