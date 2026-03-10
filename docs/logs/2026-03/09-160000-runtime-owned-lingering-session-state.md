---
title: runtime-owned lingering session state
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, transport, recovery]
---

## Summary

Moved detach-latency state from host-local recovery behavior into
`signal-runtime` transport concurrency.

## What Changed

- Extended `RuntimeTransportConcurrencySnapshot` with lingering/session-state
  detail:
  - `current_lingering_sessions`
  - `peak_lingering_sessions`
  - `current_detach_requested_sessions`
  - `current_detach_faulted_sessions`
  - per-session `state`
- Taught `signal-runtime` to update transport concurrency state from
  `record_plugin_sandbox_transport`, so `Attached`, `DetachRequested`,
  `DetachFault`, and `Detached` mutate runtime-owned session state directly.
- Added a focused runtime test proving a lingering steady-state session blocks
  new steady admission, remains visible as `DetachRequested`/`DetachFaulted`,
  and still allows one `RecoveryOverlap` replacement session.
- Added dedicated local/server deferred-teardown recovery scenarios proving a
  failed old-session teardown leaves one runtime-owned lingering
  `DetachFaulted` session in the concurrency snapshot between recovery
  attempts.
- Updated Signal docs to treat lingering detach state as part of the runtime
  control/admission model, not just an artifact of host-local failure
  injection.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_tracks_lingering_transport_sessions_as_first_class_admission_state`
- `cargo test -p signal-host-local local_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure`
- `cargo test -p signal-host-server server_host_exposes_lingering_detach_fault_state_after_deferred_recovery_teardown_failure`
- `cargo test -p signal-host-local local_host_handles_interleaved_recovery_failures_across_retries`
- `cargo test -p signal-host-server server_host_handles_interleaved_recovery_failures_across_retries`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- `transport_session_summary` remains useful for healthy-path/event-derived
  visibility, but the canonical lingering-session control state now lives in
  `transport_concurrency_snapshot`.
- The new host tests intentionally assert the runtime concurrency surface
  rather than the top-level `transport_session_summary.current_state`, because
  recovery rollback can emit later transport events for the replacement path
  even while the origin session remains lingering and faulted.

## Next Task

Drive real lingering-session cleanup through runtime-owned transport admission,
especially where a `DetachFaulted` session later completes teardown and frees
steady-state capacity across a subsequent degraded recovery attempt without a
full runtime reset.
