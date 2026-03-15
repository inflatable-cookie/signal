# g06.012 Batch 12.2 - Runtime Generic Event Snapshot Tranche

Date: 2026-03-15
Owner: core-product

## What changed

- widened `signal-plugin::PluginProcessingContract` with explicit
  `supports_note_expression` capability and aligned the CLAP, VST3, and AU
  adapter fixtures to that shared contract
- added runtime-owned `RuntimePluginEventSnapshot` to
  `signal-runtime` observation and supervisor surfaces so generic parameter,
  note, note-expression, and three-byte MIDI event output now has one bounded
  last-batch and aggregate continuity receipt
- threaded existing host-side `EventPacket::summary()` results back into
  `signal-runtime` through a new runtime-owned recording seam instead of
  leaving widened event truth in host-private payload counters only
- widened runtime discovery and capability coverage receipts so
  note-expression support is exposed alongside MIDI and note-event breadth

## Why it matters

Batch 12.1 froze the shared event vocabulary, but the widened event model was
still mostly contract-shaped. This tranche turns that into runtime-owned state:
the adapters now declare note-expression capability directly, the runtime owns
generic event continuity, and stable host delivery feeds the same Signal-owned
receipt family instead of reconstructing note-expression and MIDI depth
locally.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib runtime_plugin_event_tracking_ -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local local_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-runtime --test public_contract_boundary public_runtime_contract_boundary_is_consumable_from_reexports -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Residual risk

The widened event seam is real inside runtime and stable host-edge delivery,
but it is not yet proven as a consumer-facing boundary. Batch 12.3 still needs
to show that downstream consumers can inspect and rely on the richer event and
note-expression receipts without CLAP/VST3/AU packet reconstruction.
