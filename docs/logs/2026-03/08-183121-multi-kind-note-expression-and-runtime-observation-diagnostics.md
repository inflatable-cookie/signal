# 2026-03-08 18:31:21 - Multi-Kind Note Expression And Runtime Observation Diagnostics

Status: complete
Owner: core-product

## Summary

Broadened the brokered CLAP event path beyond the first pressure-only
note-expression slice and packaged the runtime observation surface into a
reusable diagnostics model.

This batch adds:

- reusable `RuntimeObservationDiagnostics` and `PluginFaultRecord` surfaces in
  `signal-runtime`,
- a recorder-level `diagnostics()` view so host tests and future supervisor
  tools can consume one shared observation shape,
- direct CLAP round-trip coverage for `Timbre` and `Tuning` note-expression
  events in addition to poly-pressure-derived `Pressure`,
- updated local/server supervision soak tests that consume shared observation
  diagnostics instead of rebuilding event summaries inline.

## Files

- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/lib.rs`
- `signal-runtime/src/runtime.rs`
- `signal-plugin-clap/src/lib.rs`
- `signal-host-local/src/host.rs`
- `signal-host-server/src/host.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-runtime -p signal-plugin-clap -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `effigy health`
- `effigy validate`
- `effigy test`

## Validation Notes

- Focused Rust validation is green across the CLAP adapter, runtime recorder,
  and both runtime hosts.
- Default local/server/plugin-sandbox smoke runs now report:
  - `output_events=9`
  - `parameter_events=1`
  - `parameter_gesture_events=2`
  - `parameter_modulation_events=1`
  - `note_events=1`
  - `note_expression_events=3`
  - `midi_events=1`
  - `generated_event_bytes=220`
- The runtime recorder now has direct test coverage for observation diagnostics
  summary construction and fault-detail filtering.

## Notes

- The CLAP note-expression slice now covers three expression kinds end-to-end:
  direct `Timbre`, direct `Tuning`, and MIDI poly-pressure upgraded into
  `Pressure`.
- The new runtime observation diagnostics are still mostly exercised in tests,
  but the surface is now reusable enough to be lifted into supervisor-facing
  reporting without host-specific collector code.

## Next Task

Advance the brokered CLAP path beyond the current single-block fixture by
adding richer parameter automation sequencing across multiple blocks, then
surface runtime observation diagnostics in supervisor-facing reporting or
shared fixtures so long-running engine supervision can be inspected outside the
host test modules.
