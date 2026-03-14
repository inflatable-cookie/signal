# 2026-03-08 18:22:46 - Note Expression And Runtime Observation Recorder

Status: complete
Owner: core-product

## Summary

Extended the brokered CLAP event path again so Signal now carries explicit
note-expression events alongside parameter value, parameter gesture,
parameter modulation, note, and MIDI traffic. I also replaced the duplicated
host-local runtime event collectors with the shared `RuntimeEventRecorder`
owned by `signal-runtime`.

This batch adds:

- shared `NoteExpressionEvent` support in `signal-plugin`,
- CLAP-side note-expression translation in `signal-plugin-clap`, including
  poly-pressure upgrade into a pressure note-expression event,
- shared runtime observation recording via `RuntimeEventRecorder` in
  `signal-runtime`,
- local/server host summaries and smoke output exposing
  `note_expression_events`,
- host soak tests using the shared recorder helpers instead of custom
  per-host event collectors.

## Files

- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/lib.rs`
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
- `effigy health`
- `effigy validate`
- `effigy test`

## Validation Notes

- Focused Rust validation is green across the shared event model, CLAP adapter,
  and both runtime hosts.
- Default local/server smoke runs now report:
  - `output_events=7`
  - `parameter_events=1`
  - `parameter_gesture_events=2`
  - `parameter_modulation_events=1`
  - `note_events=1`
  - `note_expression_events=1`
  - `midi_events=1`
  - `generated_event_bytes=172`
- The plugin-sandbox smoke binary reports the same seven-event output mix over
  the brokered transport.
- `effigy validate` and `effigy test` both passed after
  letting the in-flight `health` run finish; an earlier parallel attempt hit
  the expected workspace lock while `health` still owned it.

## Notes

- The current CLAP note-expression path is intentionally narrow: it upgrades
  poly-pressure into pressure note-expression and preserves the rest of the
  translated event flow.
- `RuntimeEventRecorder` now gives both hosts the same observation surface for
  supervision updates and plugin fault capture, which is the right base for
  broader supervisor diagnostics or reusable observation fixtures.

## Next Task

Broaden the CLAP event layer beyond pressure note-expression and current
gesture automation, then surface the runtime observation recorder through more
reusable supervisor diagnostics or shared fixtures so long-running supervision
verification is no longer host-test-specific.
