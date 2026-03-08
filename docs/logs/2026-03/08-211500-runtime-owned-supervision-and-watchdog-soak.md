# 2026-03-08 21:15:00 - Runtime-Owned Supervision And Watchdog Soak

Status: complete
Owner: core-product

## Summary

Moved repeated-restart watchdog supervision out of the local/server hosts and
into `signal-runtime`, then extended the host proof with a longer soak-style
recovery path that rolls across multiple lease generations.

This batch adds:

- runtime-owned supervision types and snapshot APIs in `signal-runtime`,
- runtime-owned watchdog restart accounting and safe-mode escalation,
- host integration that records watchdog restarts through `signal-runtime`
  instead of reconstructing restart escalation locally,
- longer soak tests in the local and server hosts that drive three watchdog
  restart episodes across epochs `1` through `4`,
- updated docs that treat runtime-owned supervision as the current
  implementation state rather than the next step.

## Files

- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/runtime.rs`
- `signal-runtime/src/lib.rs`
- `signal-host-local/src/host.rs`
- `signal-host-server/src/host.rs`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo run -p signal-host-local`
- `cargo run -p signal-host-server`
- `effigy validate --repo .`
- `effigy health --repo .`
- `effigy test --repo .`

## Validation Notes

- Targeted Rust tests now cover runtime-owned watchdog escalation directly in
  `signal-runtime`, plus soak recovery across multiple lease generations in the
  local and server hosts.
- Default local/server smoke runs remain clean and still report
  `watchdog_restarts=0` and `safe_mode_enabled=false` on the no-fault path.
- Signal Effigy validation still exercises the legacy C++/CMake tree rather
  than the Rust workspace. That path remained green for this batch.

## Notes

- The local/server hosts still own the recovery choreography around teardown,
  restart, and lifecycle replay, but they now defer repeated-restart counting
  and safe-mode escalation to `signal-runtime`.
- The current CLAP event layer is still intentionally lightweight. Runtime
  supervision is now in the right place, so the next batch can focus on richer
  CLAP note/parameter semantics and more adversarial mixed-fault soak paths.

## Next Task

Deepen the brokered CLAP event layer beyond the current generic
parameter/MIDI-shaped mapping, then extend the soak matrix to cover repeated
deadline misses and mixed watchdog-triggered restart episodes under the new
runtime-owned supervision state.
