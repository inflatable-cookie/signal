# 2026-03-08 19:18:32 - Monotonic Block Sequencing And Supervisor Demo

Status: complete
Owner: core-product

## Summary

Moved the Signal host fixtures from restart-aware continuity reporting into a
more realistic sustained-sequencing model by carrying a monotonic block
sequence cursor through recovery history, then added a supervisor-facing demo
outside the host `main` binaries.

This batch adds:

- `BlockSequenceContinuityReport` in `signal-plugin`,
- monotonic `next_block_sequence` carry-forward across local/server recovery
  history,
- host summaries and smoke output that expose sequence segment spans, first/last
  observed block sequence, rollover count, and gap count,
- updated soak and recovery tests that assert the new sustained sequence model,
- `RuntimeSupervisorReport::render_multiline()` and a
  `signal-runtime/examples/supervisor_report_demo.rs` example.

## Files

- `signal-plugin/src/lib.rs`
- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/runtime.rs`
- `signal-runtime/examples/supervisor_report_demo.rs`
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
- `cargo run -p signal-runtime --example supervisor_report_demo`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `cargo run -p signal-plugin-sandbox`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Validation Notes

- Host recovery tests are green with the monotonic sequence cursor carried
  through repeated watchdog restarts.
- The sequence cursor now advances even across missed-heartbeat windows, so the
  automation fixture values and final output block IDs advance with the
  sustained sequence model instead of resetting at each recovered epoch.
- The new supervisor demo renders a multiline report from `signal-runtime`
  without depending on either host `main` binary.

## Notes

- The scalar `automation_first_epoch` field still reflects legacy host
  bookkeeping rather than the earliest continuity segment epoch in every case.
  The continuity reports now carry the more precise epoch story.
- The supervisor demo currently uses synthetic recorder events rather than a
  live host soak path, but it proves the report surface works outside the host
  binaries.

## Next Task

Advance the brokered CLAP path from monotonic-sequence fixtures into runtime
timeline ownership by threading the sustained block-sequence cursor through
runtime-driven projection or scheduler state, then upgrade the supervisor
report example into a live soak-reporting tool that consumes real host output
instead of synthetic recorder events.
