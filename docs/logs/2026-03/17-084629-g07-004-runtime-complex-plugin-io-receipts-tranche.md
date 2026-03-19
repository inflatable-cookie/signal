# 2026-03-17 - g07.004 Batch 4.2 - Runtime Complex Plugin-I/O Receipts

## Summary

Batch 4.2 of `g07.004` materialized the first runtime-owned complex
plugin-I/O receipt family across discovery, execution, offline render preview,
and widened adapter baselines.

## Completed work

- added `RuntimePluginComplexIoSummary`-backed coverage to discovered plugin
  type records, plugin-format coverage, and capability coverage
- threaded complex plugin-I/O summaries through plugin-chain stage snapshots
  and offline render dependency preview
- widened VST3 and AU baseline fixtures to include multi-output instrument and
  bus-capable FX examples
- aligned local and server host scan receipts to the widened runtime-owned
  complex plugin-I/O model

## Validation

- `cargo fmt --all`
- `cargo test -p signal-plugin-vst3 -- --nocapture`
- `cargo test -p signal-plugin-au -- --nocapture`
- `cargo test -p signal-runtime runtime_plugin_chain_snapshot_reports_compensation_and_recall -- --nocapture`
- `cargo test -p signal-runtime runtime_offline_render_contract_preview_reuses_runtime_topology_tempo_clip_and_recall_contracts -- --nocapture`
- `cargo test -p signal-host-local local_host_vst3_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-local local_host_au_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_vst3_scan_and_sandbox_surface_linux_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_au_scan_and_sandbox_surface_runtime_owned_receipts -- --nocapture`

## Residual risk

Batch 4.2 closes runtime realization, not the public consumer boundary. Batch
4.3 still needs focused proof that complex plugin-I/O, multi-output
instruments, and bus-capable FX remain consumable through shared runtime,
supervisor, and stable host-edge surfaces without adapter-local pin
reconstruction.
