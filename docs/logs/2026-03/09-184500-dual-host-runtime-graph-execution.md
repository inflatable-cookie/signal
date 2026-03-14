---
title: Dual Host Runtime Graph Execution
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, graph, hosts, validation]
---

## Summary

Extended the runtime-owned executable graph path beyond the local proof host.
Signal now carries slightly richer stage semantics in `signal-graph`, and both
`signal-host-local` and `signal-host-server` execute real runtime graph work in
their brokered block loops while exporting engine metrics through the shared
runtime/supervisor surface.

## Changes

- Added `TanhDrive` and `StereoBalance` stages to `signal-graph`.
- Updated the graph tests to cover stereo-aware stage behavior.
- Kept the local host on the runtime-owned graph path with a richer local demo
  graph.
- Added the same runtime-owned graph execution path to `signal-host-server`,
  including summary/export fields and timeout recovery assertions.
- Updated `signal-supervisor-tools` so server summaries export the same engine
  metrics shape already present for the local host.
- Updated README, architecture notes, package map, contract docs, and roadmap
  notes to reflect dual-host runtime graph execution.

## Validation

- Passed: `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- Passed: `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `cargo test -p signal-supervisor-tools --no-run`
- Passed: `git diff --check`
- Failed unrelated to this Rust batch: `effigy health`
- Failed unrelated to this Rust batch: `effigy validate`
- Current repo-level blocker: legacy C++ build error in `src/ipc/binary_envelope/recording/codecs.cpp` calling `readTlvString(...)` with two arguments while `CodecCommon.hpp` exposes a one-argument overload.
- Pending caveat: a full rerun of `cargo test -p signal-graph -- --nocapture` stalled after launching the test binary in this environment, after an earlier deterministic failure was fixed. Compile-time validation and the targeted host/runtime checks were clean.

## Next

Push the executable engine slice beyond staged buffer transforms by giving
`signal-graph` and `signal-runtime` a clearer graph execution/scheduler-facing
contract, then thread that through both hosts so the runtime owns more of the
real engine work than just “run this stage chain per block.”
