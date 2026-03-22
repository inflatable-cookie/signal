# 2026-03-22 - g08.016 Batch 16.2 Linux Live Acceptance Lane

## Summary

- added the first repo-owned grouped Linux live acceptance descriptor to
  `signal-supervisor-tools` as
  `signal.runtime.linux-live-acceptance-lane`
- added the runnable Effigy lane
  `effigy acceptance:linux-live-acceptance-lane`, composed from the already-
  closed Linux live ownership, JACK coordination, PipeWire/ALSA parity, and
  Linux backend clock-topology acceptance tasks
- rolled the contract, roadmap, generation index, and feature reference
  forward so the active next step is `g08.016` Batch 16.3 consumer-proof
  closure

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_linux_live_acceptance_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools linux_live_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json`
- `effigy acceptance:linux-live-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.016` with Batch 16.3 by proving the widened Linux live
acceptance seam through shared runtime, supervisor, and stable host-edge
surfaces without introducing a daemon-local or backend-local recovery shell.
