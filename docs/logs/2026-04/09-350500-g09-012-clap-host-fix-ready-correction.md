# 09-350500 - g09.012 CLAP Host Fix Ready Correction

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md, docs/specs/batch-cards/026-g09-012-clap-host-sandbox-fix.md

## Summary

Corrected the active `g09.012` ready surface after operator clarification that
the host-side CLAP unsupported-path error must be fixed, not routed around.

## Decision

- kept `025` as a valid deferred follow-on because host comparison is still a
  real `g09.012` seam once the CLAP host gap is gone
- promoted `026` as the real next batch because the underlying host-side CLAP
  sandbox path still returns an explicit unsupported error in both local and
  server hosts
- kept the correction bounded to planning; no code changes were made in this
  step

## Evidence

- current unsupported CLAP path remains explicit in:
  - `crates/signal-host-local/src/host_api.rs`
  - `crates/signal-host-server/src/host.rs`
- the old gap is still asserted by:
  - `crates/signal-host-local/tests/public_host_edge_cross_adapter_parity.rs`
  - `crates/signal-host-server/tests/public_host_edge_cross_adapter_parity.rs`
- existing CLAP adapter/harness surfaces already provide discovery, prepare
  plan, lifecycle setup, teardown, and block protocol in `signal-plugin-clap`,
  so the seam is batch-cardable

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/026-g09-012-clap-host-sandbox-fix.md`.
