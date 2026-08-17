# g11.002 Batch 2.1 Broker Multiplexing

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/002-shared-sandbox-tier.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-dc560508`
Branch: `t3code/shared-sandbox-handoff`

## Summary

One broker child now hosts N plugin instances. DedicatedSandbox default-slot
commands still address `sandbox_id` and reject a second `load-plugin`.

## Deliverables

- `HashMap<instance_id, LoadedPlugin>` in the sandbox broker child
- instance commands: `load-plugin-instance`, `activate-instance`,
  `unload-plugin-instance`, `deactivate-instance`
- one child audio thread polling member request stamps; buffers preallocated
- `SandboxBrokerClientSession` instance-addressed wrappers
- plugin_hosting proof: two CLAP instances, two shm leases, one child

## Validation

- `cargo test -p signal-plugin-sandbox --bin signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox --test plugin_hosting -- --test-threads=1`
- `cargo test -p signal-runtime --lib sandbox_broker`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/006-g11-002-host-assembly-integration.md`.
