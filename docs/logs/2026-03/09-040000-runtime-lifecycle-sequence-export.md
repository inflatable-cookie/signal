---
title: Runtime Lifecycle Sequence Export
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, lifecycle, recovery, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so plugin-sandbox control milestones are exported as an ordered lifecycle sequence alongside recovery events.

# Changes

- Added typed plugin-sandbox lifecycle milestones to the runtime observation stream and diagnostics layer.
- Updated local and server host recovery paths to execute the CLAP teardown sequence during restart and emit lifecycle milestones for deactivate, reset, destroy, transport teardown, and sandbox restart.
- Updated the runtime supervisor report renderers and JSON export so soak tooling can inspect `lifecycle_sequence` directly.
- Tightened runtime, host, and supervisor-tool tests so lifecycle tracing is asserted through the shared event stream rather than inferred from final state.
- Updated the supervisor export contract and active docs to freeze `lifecycle_sequence` as part of the shared runtime-facing reporting surface.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

# Next Task

Deepen the runtime event stream with more of the plugin-sandbox control plane, especially typed handshake/load/create and broker attach/detach milestones, so soak analysis can correlate every recovery episode with the full lifecycle envelope around it.
