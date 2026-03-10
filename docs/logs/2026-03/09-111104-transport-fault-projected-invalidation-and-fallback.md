---
title: Transport Fault Projected Invalidation And Fallback
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-faults]
---

# Summary

Extended the canonical `transport_fault_sequence` so it now includes projected
runtime-dispatch boundaries that matter to transport-fault supervision:
detach faults, explicit invalidation, and fallback application.

# Changes

- Added `RuntimeDispatch` as a top-level `TransportFaultSource`.
- Extended `TransportFaultStage` so the canonical sequence can represent:
  - `TransportDetachFault`
  - `CompletionRegionInvalidated`
  - `LeaseEpochInvalidated`
  - `CompletionSlotInvalidated`
  - `FallbackApplied`
- Added `CompletionSlot` as a `TransportFaultResource`.
- Updated `RuntimeEventRecorder::transport_fault_events()` so it now projects:
  - `PluginSandboxTransport::DetachFault`
  - `BrokerInvalidation`
  - `CompletionSlotTransition::Invalidated`
  - `CompletionSlotTransition::FallbackApplied`
  into the canonical top-level transport-fault stream.
- Kept the specialized subordinate sequences intact:
  - `transport_sequence`
  - `invalidation_sequence`
  - `completion_slot_sequence`
  - `broker_failure_sequence`
  - `sandbox_operation_failure_sequence`
- Expanded the runtime and supervisor export fixtures so the aggregate sequence
  is pinned for host-broker, sandbox-operation, and runtime-dispatch fault
  projections together.
- Updated the README, package map, contract, and roadmap so the canonical
  transport-fault view is documented as including projected invalidation,
  fallback, and detach-fault boundaries.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether timed-out completion-slot transitions and transport
detach-requested/detached milestones should also project into the canonical
`transport_fault_sequence`, or whether the top-level view should now stop at
detach faults, invalidation, and fallback application.
