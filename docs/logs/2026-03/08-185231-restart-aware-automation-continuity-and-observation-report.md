# 2026-03-08 18:52:31 - Restart-Aware Automation Continuity And Observation Report

Status: complete
Owner: core-product

## Summary

Closed the next runtime-host verification gap by proving that the fixed CLAP
automation lane survives recovery boundaries and by making the compact runtime
observation surface reusable from `signal-runtime` instead of host-local
formatting.

This batch adds:

- restart-aware automation continuity assertions in both runtime hosts,
- explicit automation epoch coverage for timeout, crash, and heartbeat
  recovery paths,
- reusable `RuntimeObservationReport` capture/render support in
  `signal-runtime`,
- host tests that assert the shared observation report directly on mixed
  watchdog soak paths,
- dead-code suppression on the retained host observation-diagnostics helper so
  the focused host suites stay warning-clean.

## Files

- `signal-runtime/src/interfaces.rs`
- `signal-runtime/src/lib.rs`
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
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Validation Notes

- Focused Rust validation is green across the shared plugin model, CLAP broker
  path, runtime observation layer, and both runtime hosts.
- Timeout recovery preserves the automation lane's first observed epoch while
  timed-out blocks contribute no completed value/modulation payloads.
- Crash recovery and heartbeat-watchdog recovery now explicitly prove that
  automation continuity survives epoch rollover with the expected gesture
  restart edges.

## Notes

- The automation lane still uses the fixed CLAP test parameter, but the host
  assertions now verify continuity semantics rather than just per-block counts.
- `RuntimeObservationReport` is now the canonical compact observation surface
  for host consumers; the tests cover it directly on the mixed-watchdog soak
  path so the report logic is no longer only smoke-output behavior.

## Next Task

Advance the brokered CLAP path from restart-aware automation continuity into
longer-lived runtime behavior by carrying automation continuity across lease
rollover and repeated restart episodes, then expose the reusable runtime
observation report through supervisor-facing tooling or shared soak fixtures
outside the host binaries.
