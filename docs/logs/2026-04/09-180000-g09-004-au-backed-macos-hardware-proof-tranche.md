# 2026-04-09 - g09.004 AU-backed macOS hardware proof tranche

## Summary

Closed the immediate macOS proof gap after the CoreAudio inventory tranche by
moving the stable local host-edge hardware proofs onto the supported AU demo
bring-up path instead of the intentionally-deferred local CLAP sandbox path.

## Work Completed

- updated `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - added a reusable demo-plugin environment guard with serialized env mutation
  - supports booting public host-edge tests through the AU demo override path
- updated `/crates/signal-host-local/tests/public_host_edge_external_io.rs`
  - the public external-I/O and clock-topology proof lane now boots through a
    temp AU scan root and `plugin:au:instrument`
- updated `/crates/signal-host-local/tests/public_host_edge_device_supervision.rs`
  - the public device-supervision proof lane now boots through the same
    supported AU demo path
- updated `/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  - recorded `Batch 4.1 Tranche 2 Outcome`
  - checked the host-edge proof item completed in this tranche

## Validation

- `cargo test -p signal-host-local --test public_host_edge_external_io -- --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_device_supervision -- --nocapture --test-threads=1`
- `cargo test -p signal-hardware-coreaudio`
- `effigy health`

## Outcome

`g09.004` now has a stable macOS hardware proof lane that actually exercises
the new CoreAudio device truth through a runtime-owned host surface. The local
host remains explicit that CLAP sandbox ownership is deferred on that path, but
the macOS proof no longer depends on that deferred gap. The next substantive
work in this milestone is AU execution depth rather than more hardware-proof
repair.
