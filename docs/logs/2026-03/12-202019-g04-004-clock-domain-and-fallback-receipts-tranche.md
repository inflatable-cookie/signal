# 2026-03-12 - g04.004 Batch 4.2 Clock-Domain And Fallback Receipts Tranche

- Milestone: `g04.004`
- Batch: `4.2`
- Status: active

## What changed

- added backend-neutral `HardwareClockTopology` to negotiated hardware stream
  contracts in `signal-hardware`
- extended `RuntimeHostClockingSummary` with processing versus hardware sample
  rate, explicit `clock_domain`, explicit `fallback_state`, and
  `crossing_required`
- projected the new runtime-owned clock-domain/fallback receipt through
  `signal-host-local` shared observation/supervisor export without moving the
  decision into backend-local code
- restored `render_offline_with_checkpoints` trait delegation in both local and
  server hosts so the current supervisor surface stays aligned while the
  portability tranche compiles

## Focused proof

- `cargo test -p signal-hardware`
- `cargo test -p signal-hardware-coreaudio`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_topology_aware_host_io`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_cross_clock_runtime_resampling_state`
- `cargo test -p signal-host-local local_host_shared_report_tracks_device_loss_restart_failure`
- `cargo test -p signal-host-server server_host_recovers_after_crash`
- `cargo test -p signal-runtime --no-run`

## Residual risk

- aggregate or multi-clock live paths still are not exported through typed
  runtime-owned receipts
- fallback state is now explicit, but transition detail is still coarse and
  does not yet distinguish richer recovery or aggregate-clock bridge episodes
- a broader `cargo test -p signal-host-server --lib` run still hits existing
  watchdog/recovery failures unrelated to the new clock-domain receipt path

## Next Task

Continue `g04.004` with Batch 4.2 by extending the new clock-domain and
fallback receipt family to aggregate-clock topology and more explicit fallback
transition state across host paths.
