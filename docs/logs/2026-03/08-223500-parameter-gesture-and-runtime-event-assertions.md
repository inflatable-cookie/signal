# 2026-03-08 22:35:00 - Parameter Gesture And Runtime Event Assertions

Status: complete
Owner: core-product

## Summary

Extended the shared Signal event model and CLAP adapter again so the brokered
path now carries parameter-gesture semantics in addition to parameter value,
parameter modulation, note, and MIDI events. I also bound the mixed watchdog
soak path to runtime event-stream assertions instead of relying only on final
summary snapshots.

This batch adds:

- shared `ParameterGestureEvent` support in `signal-plugin`,
- CLAP-side parameter-gesture translation in `signal-plugin-clap`,
- updated host and sandbox summaries exposing gesture counts alongside the
  existing value/modulation/note/MIDI counts,
- mixed watchdog soak tests in the local and server hosts that assert emitted
  `RuntimeEvent::SupervisionChanged` and `RuntimeEvent::PluginSandboxFault`
  traffic, not just the final runtime snapshot.

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
  - `output_events=6`
  - `parameter_events=1`
  - `parameter_gesture_events=2`
  - `parameter_modulation_events=1`
  - `note_events=1`
  - `midi_events=1`
  - `generated_event_bytes=148`
- The plugin-sandbox smoke binary reports the same six-event output mix over
  the brokered transport.
- Mixed watchdog soak tests now assert:
  - three supervision updates,
  - three plugin fault events,
  - two heartbeat-watchdog fault details,
  - one deadline fault detail,
  - and the expected final watchdog trigger in both the runtime snapshot and
    the emitted supervision event stream.

## Notes

- Runtime event assertions are currently host-test-local via `EventCollector`
  helpers. They prove the right behavior, but the observation shape is not yet
  packaged as a shared reusable test fixture or supervisor diagnostics helper.
- The CLAP path still does not cover richer note-expression semantics such as
  per-note pressure, timbre, or tuning, and it still uses model-layer events
  rather than direct CLAP SDK event structs.

## Next Task

Add explicit CLAP note-expression support and broader parameter automation
surfaces on top of the current note/modulation/gesture translation, then move
the runtime-event assertion shape into reusable observation fixtures or
supervisor-facing diagnostics so this verification model is not trapped inside
host-local tests.
