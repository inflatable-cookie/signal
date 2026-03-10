---
title: Per-Session Transport Fault Freshness
date: 2026-03-09
status: closed
---

## Summary

Completed the remaining `transport_session_summary` deepening work by carrying
per-session transport-fault freshness/history into concurrent `active_sessions`
from the canonical runtime transport-fault stream.

## Changes

- extended `signal-runtime` active transport-session records with per-session
  transport-fault count plus last fault source, stage, phase, epoch, and block
- updated the transport-session summarizer to apply canonical transport-fault
  records to the matching active session instead of leaving fault visibility
  only in the top-level fault sequences
- pinned the new shape in the concurrent-session runtime fixture with mixed
  `RuntimeDispatch` and `HostBroker` fault sources
- pinned the JSON export so supervisor tooling proves per-session fault
  freshness/history serializes inside `active_sessions`
- updated the Signal README, package map, contract, and roadmap notes to treat
  `transport_session_summary` as schema-version-1 ready

## Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Next Task

Freeze `transport_session_summary` explicitly in the schema/versioning docs and
then move back to the next central engine slice, most likely hardening the
runtime/host control path around multi-session broker concurrency instead of
adding more export-surface detail.
