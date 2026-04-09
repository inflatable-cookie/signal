# 2026-04-08 - g09.002 explicit deferred gap closeout tranche

## Summary

Closed `g09.002` by turning the remaining host-facing deferred plugin gaps into
explicit runtime-owned load failures instead of silent no-op ensure behavior.

## Work Completed

- updated `/crates/signal-host-local/src/host_api.rs`
  - unsupported or undiscovered sandbox ensure requests now return explicit
    `InvalidRequest` errors
  - the attempted sandbox is still recorded in runtime-owned lifecycle state as
    a `ProtocolViolation` fault so the public host edge exposes the deferred gap
- updated `/crates/signal-host-server/src/host.rs`
  - applied the same explicit unsupported/undiscovered sandbox handling on the
    server host
- updated `/crates/signal-host-local/tests/public_host_edge_cross_adapter_parity.rs`
  - added a public CLAP deferred-gap proof showing scan truth is exported while
    the local host sandbox path answers "not supported here yet" explicitly
- updated `/crates/signal-host-server/tests/public_host_edge_cross_adapter_parity.rs`
  - added the same public CLAP deferred-gap proof on the server host
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - marked the final Batch 2.4 checkbox complete
  - changed roadmap status to `complete`
  - added `Batch 2.4 Tranche 1 Outcome`
  - advanced the next task into `g09.003`
- updated `/docs/roadmaps/g09/README.md`
  - marked `g09.002` complete in the milestone map
  - refreshed the generation next task toward `g09.003`

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

`g09.002` is now complete. The shared plugin-hosting substrate is no longer
only real in its supported scan/load/process lanes; it also answers the
remaining bounded host gaps explicitly through public scan/load receipts. That
is enough to stop widening this queue and start `g09.003`, where the remaining
work shifts from substrate and proof into real VST3 implementation depth.
