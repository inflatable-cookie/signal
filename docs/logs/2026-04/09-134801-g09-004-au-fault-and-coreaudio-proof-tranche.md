# 2026-04-09 - g09.004 AU fault and CoreAudio proof tranche

## Summary

Closed the main AU fault-explicitness seam in `g09.004` by making bounded AU
bring-up failures metadata-driven and runtime-owned, then proving that the
local host exports those AU faults alongside real CoreAudio device truth from
the same stable host surface.

## Work Completed

- updated `/crates/signal-plugin-au/src/au_host_adapter/model.rs`
  - added `AuFailureContract` to discovered and instantiated AU records
- updated `/crates/signal-plugin-au/src/au_host_adapter/introspection.rs`
  - added optional metadata-backed `init_failure`, `bus_layout_failure`, and
    `render_context_failure` parsing
- updated `/crates/signal-plugin-au/src/au_host_adapter/discovery.rs`
  - discovery now carries the parsed AU failure contract forward
- updated `/crates/signal-plugin-au/src/au_host_adapter/session.rs`
  - `instantiate_plugin(...)` now fails explicitly on initialization faults
  - `prepare_session(...)` now fails explicitly on bus-layout faults
  - `activate_instance(...)` now fails explicitly on render-context faults
- updated `/crates/signal-plugin-au/src/lib.rs`
  - added focused adapter coverage for explicit AU fault boundaries
- updated `/crates/signal-plugin-au/src/au_host_adapter/scaffold.rs`
  - kept the test-only scaffold compatible with the new failure contract
- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  - AU bring-up failures now record runtime lifecycle and fault truth before
    returning the error
- updated `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - applied the same explicit AU fault mapping on the server host path
- updated `/crates/signal-plugin-sandbox/src/broker.rs`
  - AU broker realization now respects the new fallible AU instantiate and
    activate boundaries
- updated `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - added a temp faulty AU bundle root for host-edge proof coverage
- updated `/crates/signal-host-local/tests/public_host_edge_au.rs`
  - added a local proof that boots through the real CoreAudio device path while
    a faulty AU bundle fails during render-context activation
- updated `/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  - recorded `Batch 4.2 Tranche 3 Outcome`
  - checked AU fault mapping and the first AU-plus-CoreAudio host proof item
    complete

## Validation

- `cargo test -p signal-plugin-au --lib`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_au_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_au -- --nocapture --test-threads=1`
- `effigy health`

## Outcome

`g09.004` now has explicit bounded AU failure behavior instead of only success
plumbing. The local host can export real CoreAudio device truth and AU fault
truth together from the same runtime-owned report surface, which removes the
largest remaining ambiguity about whether the macOS lane is still only a happy-
path story. The remaining work is closeout-oriented: add a focused macOS smoke
or acceptance descriptor and then decide whether the current bounded AU
execution contract is sufficient for milestone promotion.
