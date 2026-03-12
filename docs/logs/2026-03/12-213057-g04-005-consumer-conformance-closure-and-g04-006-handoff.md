# 12-213057 g04.005 Consumer Conformance Closure And g04.006 Handoff

Status: complete
Owner: core-product
Related roadmap: `docs/roadmaps/g04/005-plugin-backend-breadth-and-host-neutral-delegation-contracts.md`

## Summary

Closed `g04.005` by adding the broader Batch 5.3 conformance proofs for the
widened plugin discovery/delegation boundary and moved the active queue to
`g04.006`.

## Work Completed

- extended `crates/signal-runtime/tests/public_contract_boundary.rs` so a
  downstream-style consumer reads runtime-owned plugin discovery catalog data
  through public reexports
- added a `signal-supervisor-tools` export-consumer proof showing the widened
  discovery catalog survives runtime-owned supervisor export without
  CLAP-specific reconstruction
- updated the plugin backend/delegation contract and roadmap trail to record
  the proof boundary and the remaining explicitly deferred backend breadth
- marked `g04.005` complete and activated `g04.006`

## Validation

- `cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports`
- `cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog`

## Residual Risk

`g04.005` is closed with CLAP-first conformance, but wider adapter coverage and
broader backend-neutral capability projection remain explicitly deferred until
later work needs them.

## Next Task

Continue `g04.006` with Batch 6.1 by defining the runnable consumer
conformance matrix for the stabilized runtime/export/plugin boundary and
making that matrix usable without private implementation detail.
