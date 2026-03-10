---
title: Transport Fault Timeout And Detach Boundaries
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-faults]
---

# Summary

Extended the canonical `transport_fault_sequence` so it now includes the
remaining failure-adjacent timeout and detach lifecycle boundaries that matter
to recovery analysis, without collapsing the full healthy transport path into
the top-level fault view.

# Changes

- Added new top-level `TransportFaultStage` values for:
  - `TransportDetachRequested`
  - `TransportDetached`
  - `CompletionSlotTimedOut`
- Updated `RuntimeEventRecorder::transport_fault_events()` so the canonical
  transport-fault stream now projects:
  - `PluginSandboxTransport::DetachRequested`
  - `PluginSandboxTransport::Detached`
  - `CompletionSlotTransition::TimedOut`
- Kept normal success-path milestones such as attach and completed completion
  slots in their specialized sequences rather than promoting them into the
  canonical top-level fault surface.
- Expanded the runtime and supervisor-tool fixtures so the export is pinned for
  timeout, detach-requested, detached, detach-fault, invalidation, fallback,
  broker failure, and sandbox operation failure markers together.
- Updated the README, package map, supervisor export contract, and roadmap so
  the top-level transport-fault boundary is explicit.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether the canonical `transport_fault_sequence` should stop at
failure-adjacent timeout/detach/invalidation/fallback boundaries, or whether
it should also absorb more neutral success-path markers such as clean attach
and completed completion-slot transitions.
