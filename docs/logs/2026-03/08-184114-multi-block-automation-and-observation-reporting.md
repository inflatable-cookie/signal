# 2026-03-08 18:41:14 - Multi-Block Automation And Observation Reporting

Status: complete
Owner: core-product

## Summary

Extended the brokered CLAP fixture from isolated per-block events into a simple
multi-block automation lane and exposed the runtime observation recorder as a
compact supervisor-facing report.

This batch adds:

- shared `ParameterAutomationSummary` support in `signal-plugin`,
- a stable CLAP automation lane in `signal-plugin-clap` with one fixed
  parameter ID carrying value/modulation events across a four-block cycle,
- multi-block CLAP tests that aggregate automation behavior across the brokered
  path instead of asserting only single-block event shape,
- `RuntimeObservationDiagnostics::render_compact()` in `signal-runtime`,
- local/server host accessors and smoke output that expose the compact
  observation report directly from the shared runtime recorder.

## Files

- `signal-plugin/src/lib.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/runtime.rs`
- `signal-host-local/src/host.rs`
- `signal-host-local/src/main.rs`
- `signal-host-server/src/host.rs`
- `signal-host-server/src/main.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-plugin -p signal-plugin-clap -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `effigy health`
- `effigy validate`
- `effigy test`

## Validation Notes

- Focused Rust validation is green across the shared plugin model, CLAP broker
  path, runtime diagnostics, and both runtime hosts.
- Default local/server/plugin-sandbox smoke runs now report:
  - `output_events=11`
  - `parameter_events=2`
  - `parameter_gesture_events=2`
  - `parameter_modulation_events=2`
  - `note_events=1`
  - `note_expression_events=3`
  - `midi_events=1`
  - `generated_event_bytes=268`
- Local/server smoke output now also includes:
  - `observation=events=6 supervision_updates=0 plugin_faults=0 last_watchdog=none last_fault=none`

## Notes

- The fixed automation lane is intentionally still a fixture, but it now proves
  that one parameter can carry value/modulation continuity across multiple
  brokered blocks instead of being redefined as a fresh parameter each time.
- The compact observation report is still host-binary-facing rather than a
  standalone supervisor tool, but the rendering logic now lives in
  `signal-runtime`, not in duplicated host code.

## Next Task

Advance the brokered CLAP path from fixed multi-block automation fixtures into
longer-lived runtime behavior by carrying automation continuity across
restart/epoch boundaries, then move the compact runtime observation report into
reusable supervisor tools or shared fixtures outside the host binaries.
