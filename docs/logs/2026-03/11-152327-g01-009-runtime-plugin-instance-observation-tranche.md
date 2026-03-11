# 2026-03-11 15:23:27 GMT - g01.009 runtime plugin instance observation tranche

## Summary

Advanced `g01.009` with a meaningful follow-on `009.2` integration batch by
making host/runtime observation consume the typed sandbox instance state that
the previous control-path tranche put on the wire.

This batch still does not close `009.2` because CLAP descriptor discovery and
instance control are still fixture-backed, but plugin lifecycle transitions are
no longer inferred only from message names, heartbeat traffic, and raw fault
strings once they enter host/runtime reporting.

## What changed

- added a shared runtime-owned plugin instance state record in
  `crates/signal-runtime/src/interfaces.rs`
- added a matching runtime event and recorder/diagnostics projection so
  observation and supervisor reports can retain typed plugin lifecycle,
  readiness, processing, and last-fault metadata
- updated runtime JSON/compact report output so plugin instance state is
  exported through the shared observation surface rather than staying trapped
  inside the sandbox wire contract
- updated `crates/signal-host-local/src/host.rs` so:
  - lifecycle and heartbeat responses publish typed plugin instance state into
    runtime observation
  - local execution summaries expose the last shared plugin instance state
  - sandbox failures map runtime fault kind from the typed fault payload rather
    than reconstructing it from raw `error_kind`
- updated `crates/signal-host-server/src/host.rs` with the same typed-state
  ingestion and summary/report projection so local and server hosts stay
  aligned
- updated runtime and host tests so the new observation/export path is pinned
  at the runtime recorder seam and through one exercised host path on both
  local and server sides

## Validation

- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server --no-run`

Additional repo-owned validation was attempted after the Rust batch, but the
older full `signal-host-server` soak suite still exposes unrelated recovery
overlap contention in:

- `server_host_enters_safe_mode_after_repeated_watchdog_restarts`
- `server_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats`
- `server_host_soak_path_rolls_across_multiple_lease_generations`

Those failures are outside the typed plugin-state surface landed here, so this
tranche keeps the validation signal focused on runtime plus the touched local
and server observation paths.

## Ownership notes

- `signal-runtime` now owns the shared reporting shape for typed plugin
  instance state once sandbox/control responses cross the trust edge
- `signal-host-local` and `signal-host-server` now forward the typed state into
  runtime observation instead of only tracking coarse last-message markers
- the CLAP descriptor path is still the main remaining `009.2` contract gap,
  because the state now being exported is still sourced from fixture-backed
  discovery/control behavior

## Follow-on

The next batch should finish the remaining `009.2` discovery/control work by
replacing the fixture-only CLAP descriptor path with a concrete descriptor and
instance-control surface, then use that non-fixture control path as the bridge
into the first real plugin-backed graph/runtime render seam.
