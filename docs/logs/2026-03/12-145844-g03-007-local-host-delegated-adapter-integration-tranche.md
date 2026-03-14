# g03.007 - Local Host Delegated Adapter Integration Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed Batch 7.8 by wiring `signal-host-local` against the runtime-owned
delegated offline execution contract. The host can now prepare one concrete
delegated outcome from the runtime-authored request boundary and hand it back
to `signal-runtime` for receipt folding, output merge, artifact rewrite, and
report finalization without rebuilding offline delivery surfaces locally.

## Shipped

- added `signal-host-local` delegated offline finalization helpers that derive
  delegated stage receipts plus merged main-mix, stem, and freeze outputs from
  the runtime-owned offline render result and hand them back through
  `SignalRuntime::apply_offline_plugin_delegated_execution_outcome`
- aligned `LocalRuntimeHost` with the current runtime supervisor contract by
  delegating recording capture, media reconciliation, warp reconciliation,
  clip-processing reconciliation, and offline render entry points directly to
  `signal-runtime`
- added a focused host-local proof that forces one delegated offline plugin
  stage, runs it through the concrete host adapter, and verifies the runtime-
  owned manifest/report bundle plus exported audio artifacts are rewritten from
  the delegated outcome rather than host-local packaging code
- refreshed host-local topology assertions to match the current routed runtime
  export shape now that console grouping and bus-group counting surface more
  explicit topology detail

## Deferred

- there is still no remote/server delegated executor adapter; Batch 7.8 closes
  the requirement with one concrete host adapter only
- delegated host execution still uses a local adapter fixture rather than a
  real plugin sandbox process, so host-only plugin parity beyond this contract
  bridge remains future work if `g03.008` or later milestones need it

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Next Task

Open `g03.008` with Batch 8.1 by defining reusable profiling and soak harness
contracts that can measure the now-complete runtime-owned execution, export,
and delegated-offline substrate without shifting benchmark ownership into
hosts.
