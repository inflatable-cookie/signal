# 2026-03-17 - g07.007 Batch 7.2 - Runtime LV2 Adapter Baseline

## Summary

Batch 7.2 of `g07.007` turns LV2 from a contract-only gap into the first real
Linux-native adapter baseline across shared plugin identity, runtime-owned
discovery, and sandbox lifecycle receipts.

## Completed work

- added `PluginFormat::Lv2` to the shared plugin vocabulary and introduced the
  new `signal-plugin-lv2` adapter crate
- implemented deterministic Linux-native LV2 scan roots, bundle roots,
  manifest paths, URI identity, and bounded session-planning fixtures
- wired the server host to feed LV2 discovered-type, lifecycle, instance-state,
  transport, and parity receipts through the existing runtime-owned surfaces
- made Linux-only LV2 platform coverage explicit on runtime parity instead of
  leaving Linux plugin breadth implied
- updated roadmap, contract, package-map, and architecture reference surfaces
  to describe the realized LV2 baseline

## Validation

- `effigy test --plan --repo .`
- `cargo fmt --all`
- `cargo test -p signal-plugin-lv2 -- --nocapture`
- `cargo test -p signal-host-server server_host_lv2_scan_and_sandbox_surface_linux_runtime_owned_receipts -- --nocapture`

## Residual risk

Batch 7.2 closes runtime realization, not the public consumer boundary. Batch
7.3 still needs focused proof that Linux-native LV2 discovery, lifecycle, and
export receipts remain consumable through shared runtime, supervisor, and
stable host-edge surfaces without host-local reconstruction.
