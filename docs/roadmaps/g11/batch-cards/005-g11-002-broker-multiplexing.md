# 005 - g11.002 Broker Multiplexing

Status: complete
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.002
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/architecture/shared-sandbox-multiplexing.md, docs/roadmaps/g11/002-shared-sandbox-tier.md
Auto-start next card: yes
Depends on: 004-g11-002-multiplexing-design-note.md

## Objective

Extend `signal-plugin-sandbox` so one broker child can host N plugin instances
that share a grouping key, without changing DedicatedSandbox single-slot
behavior.

## Frozen wire

Keep existing commands. Omitted `instance_id` means `sandbox_id`.

```text
load-plugin-instance <instance_id> <library_path> <plugin_id>
activate-instance <instance_id> <sample_rate_hz> <min_frames> <max_frames>
unload-plugin-instance <instance_id>
deactivate-instance <instance_id>
```

`start-processing` / `stop-processing` stay boundary-level. v1 sequence: load
and activate each member, then start once. Do not add members after start.

Duplicate `instance_id` → existing `plugin_already_loaded`. Second default-slot
`load-plugin` stays `plugin_already_loaded`.

Internal child state: `HashMap<instance_id, LoadedPlugin>`. One audio thread
polls member request stamps. Each activate still leases its own shm region.

Client: add instance-addressed wrappers on `SandboxBrokerClientSession`. Keep
current methods as the default-instance path.

## Scope

Closed. Broker child hosts N instances. DedicatedSandbox default-slot commands
unchanged. No host factory work.

## Acceptance Criteria

- [x] two instances of the same `plugin_type_id` load, activate, and process
  through one child with two distinct shm leases
- [x] default-slot DedicatedSandbox path still rejects a second `load-plugin`
- [x] child crash remains a boundary-level receipt (member fan-out is Batch 2.3)
- [x] no new audio-thread backend

## Validation

- `cargo test -p signal-plugin-sandbox --bin signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox --test plugin_hosting -- --test-threads=1`
- `cargo test -p signal-runtime --lib sandbox_broker`

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-002-batch-2-1-broker-multiplexing.md`

## Stop Conditions

None fired.

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/006-g11-002-host-assembly-integration.md`.
