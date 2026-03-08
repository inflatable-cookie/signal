# 2026-03-08 22:15:00 - CLAP Note Semantics And Mixed Watchdog Soak

Status: complete
Owner: core-product

## Summary

Deepened the brokered CLAP event path beyond the earlier parameter-value plus
raw MIDI translation and extended the host soak matrix to cover mixed watchdog
episodes.

This batch adds:

- richer shared plugin events in `signal-plugin` for parameter modulation and
  note events,
- CLAP-side translation that upgrades note-like MIDI into explicit CLAP note
  events while preserving non-note MIDI traffic,
- brokered block tests that validate the new CLAP note/modulation-aware output
  shape instead of assuming raw event echo,
- host and sandbox summaries that now expose parameter modulation and note
  event counts in addition to parameter-value and MIDI counts,
- mixed watchdog soak coverage in the local and server hosts that alternates
  heartbeat- and deadline-triggered restarts across multiple lease generations.

## Files

- `signal-plugin/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `signal-plugin-sandbox/src/main.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`

## Validation Notes

- Focused Rust validation is green across the shared event model, CLAP adapter,
  and both runtime hosts.
- Default local/server smoke runs now report:
  - `output_events=4`
  - `parameter_events=1`
  - `parameter_modulation_events=1`
  - `note_events=1`
  - `midi_events=1`
  - `generated_event_bytes=100`
- The plugin-sandbox smoke binary reports the same four-event output mix over
  the brokered transport.
- Local/server soak coverage now includes:
  - repeated heartbeat-only restart episodes,
  - mixed heartbeat/deadline restart episodes,
  - runtime supervision snapshots that end on the expected last watchdog
    trigger after the mixed soak run.

## Notes

- The shared event model is still intentionally modest. It can now express note
  and parameter modulation semantics cleanly, but it does not yet cover richer
  CLAP note-expression, parameter gestures, or direct SDK event surfaces.
- Mixed-fault soak coverage still validates the runtime through host summaries
  and runtime snapshots, not through exhaustive event-sink assertions.

## Next Task

Add richer CLAP note-expression or parameter-gesture semantics on top of the
current note/modulation translation, then bind the mixed watchdog soak matrix
to runtime event-stream assertions so emitted `RuntimeEvent`s are verified
alongside the runtime snapshots.
