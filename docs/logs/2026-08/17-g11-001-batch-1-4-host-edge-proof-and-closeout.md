# g11.001 Batch 1.4 Host-Edge Proof And Closeout

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
Worktree: `/Users/tom/.t3/worktrees/signal/t3code-83bcb179`
Branch: `t3code/host-assembly-wiring`

## Summary

Public host-edge proof uses the same prepare → offline render path as Batch
1.3. Front doors, inventory, and crate docs now describe the factory instead
of a discovery-only host. `g11.001` is complete.

## Proof

`crates/signal-host-local/tests/public_host_edge_plugin_processor.rs`
exercises a real in-process CLAP backend from `prepare_plugin_processor`, not
broker metadata-only attach.

## Product-pull gate for `g11.002`

SharedSandbox stays deferred until a consumer names the need. Contract `014`
already owns the semantics. No research program.

## Validation

- `cargo test -p signal-host-local`
- `effigy qa:docs`
- `effigy validate`

## Next Task

Stop for operator review of the PR. Do not start `g11.002`.
