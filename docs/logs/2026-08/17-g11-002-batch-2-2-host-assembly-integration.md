# g11.002 Batch 2.2 Host Assembly Integration

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/002-shared-sandbox-tier.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-dc560508`
Branch: `t3code/shared-sandbox-handoff`

## Summary

`LocalRuntimeHost::prepare_plugin_processor(..., SharedSandbox)` routes two
prepares of the same `plugin_type_id` onto one broker child. Each prepare
returns a `ShmPluginProcessor` from its member lease. Runtime default grouping
key is `plugin:{plugin_type_id}`.

Sequential factory prepares stop the boundary before `load-plugin-instance`
because the broker still rejects load-while-processing. That is host
orchestration, not live audio-thread member add.

## Deliverables

- runtime SharedSandbox default group key `plugin:{plugin_type_id}`
- host session map keyed by grouping key; member ids `{key}:member:{n}`
- first member spawns the child; later members reuse it
- snapshot member count `max(graph_count, group_count)` so two member rows
  report `N >= 2`
- unscanned SharedSandbox fails not-discovered, not `shared_sandbox_unimplemented`

## Validation

- `cargo test -p signal-host-local --test prepare_plugin_processor`
- `cargo test -p signal-host-local --lib prepare_`
- `cargo test -p signal-runtime --lib runtime_plugin_lifecycle`

## Next Task

Execute
`docs/roadmaps/g11/batch-cards/007-g11-002-continuity-proof-and-closeout.md`.
