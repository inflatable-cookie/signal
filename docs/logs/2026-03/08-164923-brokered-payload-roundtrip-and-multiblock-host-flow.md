# 2026-03-08 16:49:23 - Brokered Payload Roundtrip And Multiblock Host Flow

Status: complete
Owner: core-product

## Summary

Moved the Signal CLAP sandbox transport from a metadata-only block proof to a
real brokered payload path. Hosts now write typed audio and event payloads into
shared memory, the sandbox reads and echoes those payloads back through the
output regions, and the local/server host shells run a short three-block
sequence rather than a single one-block fixture.

This batch adds:

- reusable `BlockPayload` helpers plus input/output payload region helpers in
  `signal-plugin`,
- CLAP protocol helpers to generate deterministic test payloads, write them
  into brokered regions, and read full block outcomes back out in
  `signal-plugin-clap`,
- sandbox-side pass-through processing that copies input audio and events into
  the negotiated output regions before committing completion state,
- multiblock host execution summaries in `signal-host-local` and
  `signal-host-server`, including block sequence, output event count, generated
  event bytes, and first output sample,
- an updated `signal-plugin-sandbox` smoke harness that exercises three
  sequential blocks over the same shared-memory lease.

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
- `cargo check -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server -p signal-plugin-sandbox`
- `cargo test -p signal-plugin -p signal-plugin-clap -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `git diff --check`

## Validation Notes

- `signal-plugin` tests now cover full audio/event payload round trips through
  input and output shared-memory regions.
- `signal-plugin-clap` tests now cover pass-through payload processing for one
  brokered block and a three-block sequence.
- The host and sandbox smoke runs now report `processed_blocks=3`,
  `output_events=2`, `generated_event_bytes=36`, and
  `first_output_sample=Some(2.0)`, confirming that brokered payload content is
  flowing through the shared-memory path rather than only metadata.

## Notes

- The current payload path is intentionally simple: the sandbox echoes generic
  events and audio back to the host without format-specific parameter or MIDI
  translation yet.
- Watchdog policy still lives outside this slice. Timeout/crash recovery works,
  but repeated heartbeat loss and repeated deadline misses do not yet drive a
  richer supervisor policy.

## Next Task

Add runtime-grade supervision around the brokered payload path: repeated miss
tracking, heartbeat watchdog policy, richer parameter/MIDI packet semantics,
and longer-running CLAP sandbox block loops that exercise the same lease and
restart flow.
