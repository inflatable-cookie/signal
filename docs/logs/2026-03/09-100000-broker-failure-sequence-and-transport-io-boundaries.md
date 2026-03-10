---
title: Broker Failure Sequence And Transport Io Boundaries
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, broker, transport, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
typed broker-side transport failures around plan creation, payload I/O, and
transport teardown instead of leaving those failures as generic resource
errors.

# Changes

- Added typed broker failure records to `signal-runtime` and exposed them
  through shared diagnostics plus `broker_failure_sequence` in text and JSON
  supervisor reports.
- Updated local and server runtime hosts to emit runtime-owned broker failure
  events when prepare-plan creation, block payload write/read, broker destroy,
  or transport teardown return I/O failures.
- Tightened runtime, host, and supervisor-tool tests so the shared report and
  export schema now pin the broker failure path explicitly.
- Updated the README, package map, roadmap, and supervisor export contract to
  freeze `broker_failure_sequence` as part of the shared runtime-facing
  reporting surface.

# Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Extend the shared runtime event stream further into brokered execution by
adding typed sandbox-side attach/flush/write protocol failures from the CLAP
harness path, so the shared supervisor report can distinguish host-visible
broker failures from sandbox-emitted transport/control faults.
