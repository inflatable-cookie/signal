# 2026-03-17 - g07.009 Linux audio backend boundary closure and g07.010 handoff

## Summary

Completed Batch 9.3 of `g07.009` by closing the bounded Linux audio backend
portability proof seam across public runtime, the stable server host edge, and
`signal-supervisor-tools`.

This tranche turns the Batch 9.2 Linux backend baseline into a real shared
consumer boundary instead of leaving ALSA, JACK, PipeWire, and unavailable
fallback meaning implied by internal runtime DTOs.

## Key changes

- added downstream-style public runtime proof that:
  - ALSA maps to `Portable`
  - JACK maps to `Guarded`
  - PipeWire maps to `Guarded`
  - missing host context remains explicit as `Unavailable` / `Unsupported`
- added stable server-host proof that the Linux-facing host edge exports the
  runtime-owned unavailable Linux backend and fallback state instead of
  inventing host-local Linux hardware capability matrices
- added the machine-readable `signal.runtime.linux-audio-backend-boundary`
  descriptor in `signal-supervisor-tools`
- wired the repo-owned `effigy acceptance:linux-audio-backend-boundary` task
- closed `g07.009` and handed the active queue to `g07.010`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_audio_backend_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_linux_audio_backend_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json`
- `effigy acceptance:linux-audio-backend-boundary --repo .`
- `effigy test --plan --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This closes the bounded Linux backend portability seam, not live ALSA, JACK,
or PipeWire host ownership, and not Linux backend clocking, duplex, or
endpoint-topology parity. Those remain explicit next-queue work in `g07.010`.

## Next Task

Continue `g07.010` with Batch 10.1 by freezing the runtime-owned Linux backend
clocking, duplex, and endpoint-topology parity contract on top of the now-
closed Linux backend portability boundary.
