# g04.001 Consumer-Facing Boundary Proof Closure

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/contracts/`, `docs/roadmaps/g04/`

## Summary

Closed `g04.001` by adding a focused downstream-style proof for the frozen
runtime/export boundary and aligning the contract/roadmap docs with that proof.

## What changed

- added `crates/signal-runtime/tests/public_contract_boundary.rs` as an
  integration test that compiles against `signal-runtime` like an external
  consumer and exercises the report/receipt boundary through public re-exports
  only
- updated contract `003` to point at that proof, keep the first stability
  promise narrow, and restate the deferred host/backend/CLI surfaces that
  remain explicitly unstable
- marked `g04.001` complete and promoted `g04.002` to the active milestone so
  the roadmap keeps one active queue

## Why this tranche

The first contract freeze was not credible until one proof showed that a
downstream consumer can use the boundary without reading `signal-runtime`
internals. This tranche provides that proof and closes the milestone cleanly
before scheduling work deepens.

## Validation

- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.002` with Batch 2.1 and define the runtime-owned multicore
scheduling contract on top of the now-explicit public/runtime boundary.
