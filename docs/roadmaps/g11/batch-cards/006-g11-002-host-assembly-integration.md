# 006 - g11.002 Host Assembly Integration

Status: complete
Owner: core-product
Updated: 2026-08-17
Master spec refs: none (baseline-routed; no active strict spec)
Roadmap refs: g11.002
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md, docs/architecture/shared-sandbox-multiplexing.md, docs/architecture/production-host-assembly-integration.md, docs/roadmaps/g11/002-shared-sandbox-tier.md
Auto-start next card: yes
Depends on: 005-g11-002-broker-multiplexing.md

## Objective

Route `PluginIsolationTier::SharedSandbox` through
`LocalRuntimeHost::prepare_plugin_processor` so two prepares of the same
`plugin_type_id` share one broker child and return two `ShmPluginProcessor`
handles.

## Frozen routing

Keep the `g11.001` factory signature. SharedSandbox:

1. require a discovered type
2. find or spawn the broker session for grouping key `plugin:{plugin_type_id}`
3. allocate a unique `instance_id`
4. `load-plugin-instance` / `activate-instance`
5. `start-processing` if the boundary is not yet running
6. `ShmPluginProcessor::attach` from that member lease
7. record runtime `placement_outcome = SharedSandbox`, grouping key, and
   `shared_boundary_member_count`

When a SharedSandbox placement rule omits `sandbox_group_key`, runtime default
must be `plugin:{plugin_type_id}`, not `sandbox:{sandbox_id}`.

Do not add a new bridge backend type. Do not grow a host-local grouping table.

## Scope

- `signal-host-local` factory + session reuse by grouping key
- `signal-runtime` SharedSandbox default grouping key
- focused host-local tests: two SharedSandbox prepares, one child, two handles
- keep DedicatedSandbox and InProcess paths unchanged
- remove `shared_sandbox_unimplemented` from the factory success path

Out of scope: terminal blast-radius proof (Batch 2.3), vendor/format grouping,
live audio-thread host pumping.

## Outcome

Two SharedSandbox prepares of the same type share one broker child and return
two `ShmPluginProcessor` handles. Runtime grouping key is
`plugin:{plugin_type_id}`; member count is `>= 2`. DedicatedSandbox and
InProcess factory paths are unchanged. Unscanned SharedSandbox fails
not-discovered.

## Acceptance Criteria

- [x] two SharedSandbox prepares of the same type share one broker child
- [x] each prepare returns a working `RenderPluginProcessor` from a real lease
- [x] runtime snapshot shows grouping key and member count `>= 2`
- [x] DedicatedSandbox / InProcess factory tests still pass
- [x] `PluginIsolationTier::SharedSandbox` no longer returns
  `shared_sandbox_unimplemented`

## Validation

- `cargo test -p signal-host-local --test prepare_plugin_processor`
- `cargo test -p signal-host-local --lib prepare_`
- focused runtime placement tests for the SharedSandbox default group key

## Evidence Required

- batch log: `docs/logs/2026-08/17-g11-002-batch-2-2-host-assembly-integration.md`

## Stop Conditions

- factory signature change appears necessary
- SharedSandbox seems to need a new `PluginBlockProcessor`
- grouping cannot be expressed through existing runtime placement receipts

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/007-g11-002-continuity-proof-and-closeout.md`.
