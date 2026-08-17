# g11.002 Batch 2.3 Continuity Proof And Closeout

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/002-shared-sandbox-tier.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-dc560508`
Branch: `t3code/shared-sandbox-handoff`

## Summary

Killing a SharedSandbox broker child fans the crash onto every sandbox that
shares the grouping key. IsolatedSandbox does not fan out. Runtime receipts
stay the authority; the host does not keep a private process map.

First crash maps to `Faulted` / Restartable (`quarantine_after_faults` is 2).
Contract `014` allows Restartable or Terminal. Isolation is proved by
state/fault, not class alone: IsolatedSandbox peers stay Stopped with no
`last_fault_kind`.

`g11.002` is complete. SharedSandbox is no longer unimplemented.

## Proof

- `shared_sandbox_fault_fans_out_to_every_group_member`
- `isolated_sandbox_fault_does_not_fan_out`
- `prepare_plugin_processor_shared_sandbox_child_crash_fans_out`

Host helper `crash_shared_sandbox_broker_child` kills the child for
`plugin:{plugin_type_id}` and records `PluginFaultKind::Crash` /
`shared_boundary_child_dead` on one member; runtime fans out.

## Docs

Inventory, host-assembly map, Contract `072`, and `g11` front doors describe
the landed SharedSandbox path. Remaining deferred scope is product UX and
vendor/format grouping, not the shared-boundary tier.

## Validation

- `cargo test -p signal-runtime --lib runtime_plugin_lifecycle`
- `cargo test -p signal-host-local --test prepare_plugin_processor`
- `cargo test -p signal-host-local --lib prepare_`
- `effigy qa:docs`
- `effigy validate`

## Next Task

Stop for operator review of the `g11.002` PR. Do not start a follow-on
generation.
