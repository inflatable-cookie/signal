# 2026-03-08 20:10:00 - Brokered Block Processing Heartbeat And Teardown

Status: complete
Owner: core-product

## Summary

Moved the Signal CLAP sandbox path from brokered region setup into a real
single-block processing proof: the host now writes block-dispatch data into the
mapped shared-memory region, the sandbox reads and completes that block through
the same region, and the control plane now includes heartbeat plus orderly
deactivate/reset/destroy messages.

This batch adds:

- plugin-domain control messages for heartbeat, deactivate, reset, and destroy
  in `signal-ipc`,
- generic shared-memory region helpers and block/completion serialization in
  `signal-plugin`,
- CLAP helpers to write a brokered block dispatch, read completion state, send
  heartbeat, and build teardown sequences in `signal-plugin-clap`,
- sandbox-side CLAP handling for heartbeat, deactivate, reset, destroy, normal
  block completion, and deadline-miss marking,
- host-level use of that brokered block-processing path in both
  `signal-host-local` and `signal-host-server`,
- smoke output that now surfaces heartbeat and completion state rather than
  only lifecycle-envelope counts.

## Files

- `signal-ipc/src/lib.rs`
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
- `cargo check -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server -p signal-plugin-sandbox`
- `cargo test -p signal-ipc -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`
- `effigy health`
- `effigy validate`
- `effigy test`

## Validation Notes

- The focused Rust test slice now covers:
  - brokered block dispatch encode/decode,
  - completion-slot round trips,
  - CLAP heartbeat plus deactivate/reset/destroy sequencing,
  - brokered block completion and timeout marking,
  - local/server timeout and crash recovery with a completed post-restart block.
- The smoke runs now report `heartbeat_responses=1`, `processed_blocks=1`, and
  `completion=Completed`, confirming the default host path exercises the
  brokered transport beyond prepare/activate.

## Notes

- The current proof writes header/render-context/completion metadata into the
  shared region, not full audio/event payloads yet.
- The timeout path is now exercised through the completion region, while crash
  recovery still enters from a supervisor-side failure event rather than a dead
  transport/heartbeat loop.

## Next Task

Extend the brokered processing proof into a reusable transport path: move real
audio/event payloads through the shared regions, add watchdog policy around
repeated misses/heartbeats, and thread parameter/MIDI exchange plus multi-block
sequencing through the same CLAP sandbox transport.
