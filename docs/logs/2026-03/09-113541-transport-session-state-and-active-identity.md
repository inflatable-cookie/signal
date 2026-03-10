---
title: Transport Session State And Active Identity
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-session]
---

# Summary

Upgraded `transport_session_summary` from a counter-only healthy-path view into
a real session-state surface by adding explicit current attachment state and
active identity fields.

# Changes

- Added `TransportSessionState` to `signal-runtime`.
- Extended `TransportSessionSummary` with:
  - `current_state`
  - `currently_attached`
  - `active_sandbox_id`
  - `active_lease_id`
  - `active_region_id`
- Derived those fields by replaying the existing transport sequence, so the
  session summary now distinguishes current attachment state from merely “last
  observed” identity.
- Kept the existing `last_*` identity fields so tooling can still inspect the
  most recent transport record even after detachment/fault.
- Updated compact, multiline, and JSON rendering so the stateful session
  summary is visible through the shared supervisor report surface.
- Re-exported `TransportSessionState` from `signal-runtime`.
- Updated runtime and supervisor-tool fixtures so the stateful session summary
  shape is pinned in both direct runtime tests and export tests.
- Updated the README, package map, contract, and roadmap so the healthy-path
  session companion is documented as stateful rather than counter-only.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether `transport_session_summary` should now be treated as stable for
schema version 1, or whether it still needs one more round of state detail
such as in-flight dispatch or heartbeat freshness before the split can be
considered frozen.
