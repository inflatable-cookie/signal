---
title: Transport Fault Phase Resource Metadata
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-faults]
---

# Summary

Deepened the canonical `transport_fault_sequence` so it now carries useful
top-level phase, resource, and operation metadata instead of forcing tooling
to drop into the broker-specific and sandbox-specific subordinate sequences
for basic interpretation.

# Changes

- Added `TransportFaultPhase` and `TransportFaultResource` to the shared
  runtime interface surface.
- Extended `TransportFaultRecord` with typed `phase` and `resource` fields.
- Replaced the generic broker-side top-level `operation="broker"` value with
  concrete broker operation names such as `prepare_plan.create`,
  `block_payload.write`, `block_payload.read`, `lease.destroy_region`, and
  `lease.teardown_transport`.
- Kept sandbox-derived top-level `operation` values sourced from the CLAP
  harness path, while adding shared phase/resource mapping for prepare,
  dispatch, and control-path failures.
- Updated runtime and supervisor-tool fixture coverage so the top-level export
  is now pinned for `source`, `phase`, `resource`, and `operation`.
- Updated the README, package map, supervisor export contract, and roadmap so
  the canonical transport-fault surface is documented as more than a flat
  source-labeled merge.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether completion-slot invalidation, fallback application, and broker
detach faults should also project into the canonical `transport_fault_sequence`,
or remain only in their specialized subordinate sequences.
