# 09-360500 - g09.012 CLAP Host Fix Closeout And Host Comparison Reactivation

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/026-g09-012-clap-host-sandbox-fix.md, docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md

## Summary

Closed the bounded CLAP host sandbox repair batch, then reactivated the deferred
host comparison card because that wrapper is no longer blocked by a known CLAP
capability gap.

## Work Completed

- fixed the local host module surface so the existing CLAP sandbox helper is
  actually imported into the real `ensure_plugin_sandbox(...)` and restart path
- gave the default local and server demo assemblies explicit CLAP plugin ids so
  `boot_default()` exercises the real CLAP path instead of failing on missing
  plugin identity
- updated the public local and server cross-adapter parity proofs to assert
  bounded CLAP lifecycle truth:
  - lifecycle stage reaches `TransportAttached`
  - sandbox state is `Ready`
  - active transport is true
  - no protocol-violation fault is recorded
- updated the strict currentness/front-door surfaces so `026` is complete and
  `025` is again the active ready card

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity local_shared_host_edge_exports_bounded_clap_sandbox_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity server_shared_host_edge_exports_bounded_clap_sandbox_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo run -q -p signal-host-local`
- `cargo run -q -p signal-host-server`
- `effigy health`

## Outcome

- the explicit CLAP unsupported-path behavior is no longer the blocking seam for
  host bring-up
- both host binaries boot successfully on the real CLAP path
- host comparison is now the next honest `g09.012` batch; it is no longer
  deferred behind a missing CLAP host capability

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md`.
