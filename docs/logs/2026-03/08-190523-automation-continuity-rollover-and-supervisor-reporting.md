# 2026-03-08 19:05:23 - Automation Continuity Rollover And Supervisor Reporting

Status: complete
Owner: core-product

## Summary

Extended the Signal engine proof from basic restart-aware automation accounting
into shared continuity reporting and reusable supervisor-facing observation
capture.

This batch adds:

- `AutomationContinuityReport` in `signal-plugin` so host/runtime paths can
  track automation by processing epoch and lease rather than only aggregate
  counts,
- inter-episode continuity blocks in the local/server soak paths so repeated
  restart scenarios now exercise automation continuity across multiple epochs
  and lease generations,
- `RuntimeSupervisorReport` in `signal-runtime` so a supervisor-facing report
  can be captured from runtime state plus the event recorder without depending
  on host-`main` formatting,
- host summaries and smoke output that expose automation segment counts,
  segment epochs, and lease rollovers,
- stronger local/server tests covering timeout, crash, heartbeat, safe-mode,
  and mixed watchdog recovery with continuity assertions.

## Files

- `signal-plugin/src/lib.rs`
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

- Shared plugin/runtime/host tests are green with the new continuity and
  supervisor-report surfaces.
- Repeated-restart soak paths now carry successful automation-bearing blocks
  between recovery episodes instead of proving only final-epoch behavior.
- The mixed watchdog path shows the important nuance that the continuity
  segment tracker preserves epoch-level rollout while the older
  `automation_first_epoch` scalar still reflects later runtime bookkeeping.

## Notes

- The continuity tracker is intentionally shared in `signal-plugin` because the
  same semantics will matter for any host/runtime assembly, not only the local
  and server host shells.
- `RuntimeSupervisorReport` is still a report/fixture surface rather than a
  standalone supervisor tool, but the report capture is no longer trapped in
  host smoke output.

## Next Task

Advance the brokered CLAP path from continuity reporting into sustained runtime
sequencing by keeping automation and block-sequence continuity explicit across
lease rollover and repeated restart episodes, then surface the supervisor
report through reusable soak fixtures or a dedicated supervisor-facing tool
outside the host binaries.
