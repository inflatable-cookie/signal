# 2026-03-19 16:30:44 - g08.003 PipeWire/ALSA parity boundary closure

## Summary

Closed `g08.003` by turning the new PipeWire/ALSA runtime receipt family into
a repo-owned consumer boundary.

## What changed

- added `signal.runtime.pipewire-alsa-parity-boundary` to
  `signal-supervisor-tools`
- added `effigy acceptance:pipewire-alsa-parity-boundary`
- proved the widened parity seam through:
  - public runtime export
  - stable local host edge
  - stable server host edge
  - machine-readable supervisor boundary descriptor
- updated the contract, roadmap, architecture reference, and generation
  pointers to mark `g08.003` complete

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_pipewire_alsa_parity_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools pipewire_alsa_parity_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-pipewire-alsa-parity-boundary --format=json`
- `effigy acceptance:pipewire-alsa-parity-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This closes the bounded consumer seam, not full PipeWire daemon, portal, ALSA
reservation, or distro-policy depth. Those remain later Linux workflow or
acceptance work, not part of `g08.003`.

## Next Task

Open `g08.004` with Batch 4.1 by freezing the first runtime-owned LV2 worker,
URID, patch, and extension-negotiation contract on top of the now-closed live
Linux ownership, JACK coordination, and PipeWire/ALSA parity seams.
