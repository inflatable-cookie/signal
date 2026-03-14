# 2026-03-12 - g04.004 Portability Closure And Plugin Boundary Handoff

- Milestone: `g04.004`
- Status: complete

## What closed

- completed the hardware portability milestone by extending the existing
  runtime-owned host clocking receipt family with explicit transition-state
  export on top of the already-added clock-domain and fallback-state export
- proved aggregate-clock entry and return-to-direct recovery on the shared host
  observation/supervisor path
- recorded the intentionally deferred backend breadth explicitly:
  multi-member aggregate detail, drift compensation, and broader backend-matrix
  coverage remain out of scope until a consumer actually needs them

## Validation

- `cargo fmt --all`
- `cargo test -p signal-hardware`
- `cargo test -p signal-hardware-coreaudio`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_topology_aware_host_io`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_cross_clock_runtime_resampling_state`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_aggregate_clock_domain`
- `cargo test -p signal-host-local local_host_shared_report_tracks_return_to_direct_after_cross_clock_fallback`
- `cargo test -p signal-host-local local_host_shared_report_tracks_device_loss_restart_failure`
- `cargo test -p signal-host-server server_host_recovers_after_crash`
- `cargo test -p signal-runtime --no-run`
- `git diff --check`
- `effigy health`

## Residual risk

- `signal-host-server` still has broader watchdog/recovery test failures outside
  this hardware portability receipt path
- the portability contract is now usable, but only one concrete host backend
  path is exercised end-to-end

## Next Task

Open `g04.005` with Batch 5.1 and define the format-neutral plugin backend and
host-neutral delegation contract on top of the closed runtime, hardware, and
deferred-work boundaries.
