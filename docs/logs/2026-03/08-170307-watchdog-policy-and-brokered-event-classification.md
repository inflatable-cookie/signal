# 2026-03-08 17:03:07 - Watchdog Policy And Brokered Event Classification

Status: complete
Owner: core-product

## Summary

Added the first real supervision policy layer on top of the brokered CLAP
payload path. Signal now has a shared sandbox watchdog state machine for
consecutive deadline and heartbeat misses, hosts exercise that watchdog during
brokered block execution, and brokered event payloads are now classified into
parameter-value vs MIDI traffic rather than being treated as an undifferentiated
event count.

This batch adds:

- `SandboxWatchdogPolicy`, `SandboxWatchdogState`, and restart-trigger outcomes
  in `signal-plugin`,
- `EventPacketSummary` helpers in `signal-plugin` so parameter-value and MIDI
  traffic can be counted separately,
- host-side heartbeat polling per block in `signal-host-local` and
  `signal-host-server`,
- watchdog-triggered recovery for consecutive timeout and heartbeat-miss paths,
- richer host summaries that report parameter events, MIDI events, deadline
  misses, heartbeat misses, and watchdog trigger reason,
- heartbeat-watchdog recovery tests for both local and server hosts.

## Files

- `signal-plugin/src/lib.rs`
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

- The local and server host tests now cover three recovery cases:
  - timeout watchdog trigger,
  - heartbeat watchdog trigger,
  - crash recovery.
- Default host smoke runs now report:
  - `control_requests=9`
  - `heartbeat_responses=4`
  - `processed_blocks=3`
  - `parameter_events=1`
  - `midi_events=1`
  - `watchdog_triggered=false`
- The sandbox shell smoke output now reports the same parameter/MIDI split as
  the host path.

## Notes

- The new watchdog policy is threshold-based and restart-oriented. It does not
  yet drive a richer runtime degraded-mode or safe-mode transition after
  repeated restarts.
- Event semantics are still generic enough to stay format-neutral; this batch
  classifies parameter-value vs MIDI payloads but does not yet translate them
  into CLAP-specific event structures.

## Next Task

Move from threshold counting to sustained runtime policy: run longer brokered
block loops over the same lease, escalate repeated watchdog-triggered restarts
into degraded or safe-mode runtime behavior, and begin replacing the generic
parameter/MIDI packet helpers with CLAP-specific event translation.
