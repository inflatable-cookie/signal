# 2026-03-08 17:36:57 - Runtime Safe-Mode Escalation And CLAP Event Translation

Status: complete
Owner: core-product

## Summary

Extended the Signal sandbox transport from basic watchdog recovery into a more
credible runtime policy proof. Hosts now run longer steady-state brokered block
loops over the same lease, repeated watchdog-triggered restarts can escalate
into runtime safe-mode/degraded readiness, and the CLAP adapter now owns an
explicit translation boundary between generic brokered parameter/MIDI packets
and CLAP-oriented event representations.

This batch adds:

- shared restart-escalation policy state in `signal-plugin`,
- longer steady-state brokered block loops in the local/server hosts,
- host-side escalation from repeated heartbeat watchdog restarts into runtime
  safe mode,
- CLAP event translation helpers in `signal-plugin-clap`,
- host tests for repeated watchdog-triggered restart escalation into degraded
  runtime readiness,
- updated host and sandbox smoke output showing the longer block run and the
  new watchdog restart/safe-mode fields.

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
- `cargo check -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server -p signal-plugin-sandbox`
- `cargo test -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`

## Validation Notes

- Default local/server host smoke runs now report:
  - `control_requests=14`
  - `heartbeat_responses=9`
  - `processed_blocks=8`
  - `last_block_sequence=7`
  - `parameter_events=1`
  - `midi_events=1`
  - `watchdog_restarts=0`
  - `safe_mode_enabled=false`
- The host test matrix now covers:
  - timeout watchdog recovery,
  - heartbeat watchdog recovery,
  - crash recovery,
  - repeated heartbeat watchdog restarts escalating into degraded/safe-mode
    runtime state.
- `signal-plugin-clap` now includes an explicit event-translation round-trip
  test in addition to the existing brokered block transport coverage.

## Notes

- Safe-mode escalation is still host-owned. This batch proves the policy shape,
  but repeated-restart degradation is not yet centralized inside
  `signal-runtime`.
- The CLAP translation layer is still intentionally lightweight: it maps
  generic parameter-value and MIDI packets into CLAP-oriented event structs, but
  does not yet cover richer CLAP note/parameter semantics or direct SDK
  integration.

## Next Task

Move repeated-restart supervision policy closer to `signal-runtime`, add
longer-running brokered soak/fault loops that exercise lease rollover over
multiple restart generations, and deepen the CLAP event layer beyond the
current generic parameter/MIDI mapping into more concrete host/plugin event
semantics.
